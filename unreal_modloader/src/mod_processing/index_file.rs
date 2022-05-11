use std::collections::HashMap;
use std::thread;

use log::{debug, warn};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use unreal_modintegrator::metadata::DownloadInfo;

use crate::error::ModLoaderWarning;
use crate::game_mod::{GameModVersion, SelectedVersion};
use crate::version::Version;
use crate::ModLoaderAppData;

use super::verify;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexFile {
    mods: HashMap<String, IndexFileMod>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IndexFileMod {
    latest_version: String,
    versions: HashMap<String, IndexFileModVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub(crate) struct IndexFileModVersion {
    download_url: String,
    filename: String,
}

pub(crate) fn gather_index_files(
    data: &mut ModLoaderAppData,
    filter: &Vec<String>,
) -> HashMap<String, DownloadInfo> {
    let mut index_files: HashMap<String, DownloadInfo> = HashMap::new();

    for (mod_id, game_mod) in data.game_mods.iter() {
        if game_mod.download.is_some() && filter.contains(mod_id) {
            let download_info = game_mod.download.as_ref().unwrap();
            index_files.insert(mod_id.clone(), download_info.clone());
        }
    }

    index_files
}

pub(crate) fn download_index_files(
    index_files_info: HashMap<String, DownloadInfo>,
) -> (HashMap<String, IndexFileMod>, Vec<ModLoaderWarning>) {
    let handles = index_files_info
        .into_iter()
        .map(|(mod_id, download_info)| {
            thread::spawn(move || {
                let client = Client::new();
                let response = client.get(download_info.url.as_str()).send();
                if response.is_err() {
                    warn!(
                        "Failed to download index file for {:?}, {}",
                        mod_id,
                        response.unwrap_err()
                    );

                    return Err(ModLoaderWarning::index_file_download_failed(mod_id));
                }

                let response = response.unwrap();
                if !response.status().is_success() {
                    warn!(
                        "Failed to download index file for {:?}, {}",
                        mod_id,
                        response.status()
                    );

                    return Err(ModLoaderWarning::index_file_download_failed(mod_id));
                }

                let index_file =
                    serde_json::from_str::<IndexFile>(response.text().unwrap().as_str());

                if index_file.is_err() {
                    warn!(
                        "Failed to parse index file for {}: {}",
                        mod_id,
                        index_file.unwrap_err()
                    );

                    return Err(ModLoaderWarning::invalid_index_file(mod_id));
                }
                let index_file = index_file.unwrap();

                let index_file_mod = index_file.mods.get(&mod_id);

                if index_file_mod.is_none() {
                    warn!("Index file for {} does not contain that mod", mod_id);

                    return Err(ModLoaderWarning::index_file_missing_mod(mod_id));
                }

                return Ok((mod_id, index_file_mod.unwrap().clone()));
            })
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();

    let index_file_data = handles
        .into_iter()
        // for general thread errors
        .filter_map(|handle| {
            handle
                .join()
                .map_err(|err| {
                    warn!("error joining thread: {:?}", err);
                })
                .ok()
        })
        // for download errors
        .filter_map(|download_result| {
            download_result
                .map_err(|err| {
                    warn!("error downloading index file: {:?}", err);
                    warnings.push(err);
                })
                .ok()
        })
        .collect::<Vec<_>>();

    let mut index_files: HashMap<String, IndexFileMod> = HashMap::new();
    for (mod_id, download_info) in index_file_data.iter() {
        index_files.insert(mod_id.to_owned(), download_info.to_owned());
    }

    (index_files, warnings)
}

pub(crate) fn insert_index_file_data(
    index_files: &HashMap<String, IndexFileMod>,
    data: &mut ModLoaderAppData,
) -> Vec<ModLoaderWarning> {
    let mut warnings = Vec::new();

    for (mod_id, index_file) in index_files.iter() {
        let game_mod = data.game_mods.get_mut(mod_id).unwrap();

        for (version_raw, version_info) in index_file.versions.iter() {
            let version = Version::try_from(version_raw);
            if version.is_err() {
                warn!(
                    "Failed to parse version {:?} from index file for mod {:?}",
                    version_raw, mod_id
                );
                warnings.push(ModLoaderWarning::invalid_index_file(mod_id.to_owned()));

                continue;
            }

            if !verify::verify_mod_file_name(&version_info.filename) {
                warn!(
                    "Failed to verify filename {:?} from index file for mod {:?}",
                    version_info.filename, mod_id
                );
                warnings.push(ModLoaderWarning::invalid_index_file(mod_id.to_owned()));

                continue;
            }

            if game_mod.versions.contains_key(&version.as_ref().unwrap()) {
                let mut existing_version_data = game_mod
                    .versions
                    .get_mut(&version.as_ref().unwrap())
                    .unwrap();

                existing_version_data.download_url = Some(version_info.download_url.clone());
            } else {
                game_mod.versions.insert(
                    version.unwrap(),
                    GameModVersion {
                        file_name: version_info.filename.clone(),
                        downloaded: false,
                        download_url: Some(version_info.download_url.clone()),
                        metadata: None,
                    },
                );
            }
        }

        let latest_version = Version::try_from(&index_file.latest_version);
        if latest_version.is_err() {
            warn!(
                "Failed to parse version {:?} from index file for mod {:?}",
                index_file.latest_version, mod_id
            );
            warnings.push(ModLoaderWarning::invalid_index_file(mod_id.to_owned()));

            continue;
        }

        match game_mod.selected_version {
            SelectedVersion::Latest(_) => {
                game_mod.selected_version =
                    SelectedVersion::Latest(latest_version.as_ref().unwrap().clone());
            }
            SelectedVersion::LatestIndirect(_) => {
                game_mod.selected_version =
                    SelectedVersion::Latest(latest_version.as_ref().unwrap().clone());
            }
            SelectedVersion::Specific(_) => {}
        }

        game_mod.latest_version = Some(latest_version.unwrap());

        debug!("Loaded index file for {}", mod_id);
    }

    warnings
}
