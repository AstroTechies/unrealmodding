use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use unreal_modintegrator::metadata::DownloadInfo;

use crate::game_mod::{GameModVersion, SelectedVersion};
use crate::version::Version;
use crate::AppData;

pub(crate) fn gather_index_files(data: &mut AppData) -> HashMap<String, DownloadInfo> {
    let mut index_files: HashMap<String, DownloadInfo> = HashMap::new();

    for (mod_id, game_mod) in data.game_mods.iter() {
        let download_info = game_mod.download.clone();
        if let Some(download_info) = download_info {
            index_files.insert(mod_id.to_owned(), download_info);
        }
    }

    index_files
}

pub(crate) fn download_index_files(
    index_files_info: HashMap<String, DownloadInfo>,
) -> HashMap<String, IndexFileMod> {
    let mut index_files: HashMap<String, IndexFileMod> = HashMap::new();

    let client = Client::new();

    for (mod_id, download_info) in index_files_info.iter() {
        let response = client.get(download_info.url.as_str()).send();
        if response.is_err() {
            println!(
                "Failed to download index file for {}: {}",
                mod_id,
                response.unwrap_err()
            );

            continue;
        }

        let index_file =
            serde_json::from_str::<IndexFile>(response.unwrap().text().unwrap().as_str());

        if index_file.is_err() {
            println!(
                "Failed to parse index file for {}: {}",
                mod_id,
                index_file.unwrap_err()
            );

            continue;
        }
        let index_file = index_file.unwrap();

        let index_file_mod = index_file.mods.get(mod_id);

        if index_file_mod.is_none() {
            println!("Index file for {} does not contain that mod", mod_id);

            continue;
        }

        index_files.insert(mod_id.to_owned(), index_file_mod.unwrap().to_owned());
    }

    index_files
}

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

pub(crate) fn insert_index_file_data(
    index_files: &HashMap<String, IndexFileMod>,
    data: &mut AppData,
) {
    for (mod_id, index_file) in index_files.iter() {
        let game_mod = data.game_mods.get_mut(mod_id).unwrap();

        for (version, version_info) in index_file.versions.iter() {
            game_mod.versions.insert(
                Version::try_from(version).unwrap(),
                GameModVersion {
                    file_name: version_info.filename.clone(),
                    downloaded: true,
                    download_url: Some(version_info.download_url.clone()),
                    metadata: None,
                },
            );
        }

        game_mod.selected_version =
            SelectedVersion::Latest(Version::try_from(&index_file.latest_version).unwrap());

        println!("Loaded index file for {}", mod_id);
    }
}
