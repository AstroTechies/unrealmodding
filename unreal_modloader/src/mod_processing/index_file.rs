use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;
use std::thread;

use log::{debug, warn};
use reqwest::blocking::Client;
use semver::Version;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use unreal_modmetadata::DownloadInfo;

use crate::error::ModLoaderWarning;
use crate::game_mod::{GameModVersion, SelectedVersion};
use crate::ModLoaderAppData;

use super::verify;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct IndexFile {
    mods: HashMap<String, IndexFileMod>,
}

fn string_to_version<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr<Err = semver::Error>,
    D: Deserializer<'de>,
{
    struct StringDeserializer<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringDeserializer<T>
    where
        T: FromStr<Err = semver::Error>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string")
        }
        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }
    }
    deserializer.deserialize_any(StringDeserializer(PhantomData))
}

fn deserialize_version_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<Version, IndexFileModVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Hash, PartialEq, Eq, Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "string_to_version")] Version);
    let a: HashMap<Wrapper, IndexFileModVersion> = HashMap::deserialize(deserializer)?;
    Ok(a.into_iter().map(|(Wrapper(k), v)| (k, v)).collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct IndexFileMod {
    #[serde(deserialize_with = "string_to_version")]
    pub latest_version: Version,
    #[serde(deserialize_with = "deserialize_version_map")]
    pub versions: HashMap<Version, IndexFileModVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Hash)]
pub(crate) struct IndexFileModVersion {
    pub download_url: String,
    #[serde(rename = "filename")]
    pub file_name: String,
}

impl IndexFileModVersion {
    pub fn new(download_url: String, file_name: String) -> Self {
        IndexFileModVersion {
            download_url,
            file_name,
        }
    }
}

pub(crate) fn gather_index_files(
    data: &mut ModLoaderAppData,
    filter: &[String],
) -> HashMap<String, DownloadInfo> {
    //let mut index_files: HashMap<String, DownloadInfo> = HashMap::new();

    data.game_mods
        .iter()
        .filter(|(mod_id, _)| filter.contains(mod_id))
        .filter_map(|(mod_id, game_mod)| {
            game_mod
                .download
                .as_ref()
                .map(|download_info| (mod_id.clone(), download_info.clone()))
        })
        .collect()
}

pub(crate) fn download_index_file(
    mod_id: String,
    download_info: &DownloadInfo,
) -> Result<(String, IndexFileMod), ModLoaderWarning> {
    let client = Client::new();
    let response = client.get(download_info.url.as_str()).send();
    if let Err(err) = response {
        warn!("Failed to download index file for {:?}, {}", mod_id, err);

        return Err(ModLoaderWarning::index_file_download_failed(mod_id, err));
    }

    let response = response.unwrap();
    if !response.status().is_success() {
        warn!(
            "Failed to download index file for {:?}, {}",
            mod_id,
            response.status()
        );

        return Err(ModLoaderWarning::index_file_download_failed_status(
            mod_id,
            response.status(),
        ));
    }

    let index_file =
        serde_json::from_str::<IndexFile>(response.text().unwrap().as_str()).map_err(|err| {
            warn!("Failed to parse index file for {}: {}", mod_id.clone(), err);
            ModLoaderWarning::invalid_index_file(mod_id.clone())
        })?;

    match index_file.mods.get(&mod_id) {
        Some(index_file_mod) => Ok((mod_id, index_file_mod.clone())),
        None => {
            warn!("Index file for {} does not contain that mod", mod_id);
            Err(ModLoaderWarning::index_file_missing_mod(mod_id))
        }
    }
}

pub(crate) fn download_index_files<I>(
    index_files_info: I,
) -> (HashMap<String, IndexFileMod>, Vec<ModLoaderWarning>)
where
    I: IntoIterator<Item = (String, DownloadInfo)>,
{
    // we need to collect to allow multi threading to actually happen
    #[allow(clippy::needless_collect)]
    let handles = index_files_info
        .into_iter()
        .map(|(mod_id, download_info)| {
            thread::spawn(move || download_index_file(mod_id, &download_info))
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();

    (
        handles
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
            .collect(),
        warnings,
    )
}

pub(crate) fn insert_index_file_data(
    index_files: &HashMap<String, IndexFileMod>,
    data: &mut ModLoaderAppData,
) -> Vec<ModLoaderWarning> {
    let mut warnings = Vec::new();

    for (mod_id, index_file) in index_files.iter() {
        let game_mod = data.game_mods.get_mut(mod_id).unwrap();

        for (version, version_info) in index_file.versions.iter() {
            if !verify::verify_mod_file_name(&version_info.file_name) {
                warn!(
                    "Failed to verify filename {:?} from index file for mod {:?}",
                    version_info.file_name, mod_id
                );
                warnings.push(ModLoaderWarning::invalid_index_file(mod_id.to_owned()));

                continue;
            }

            if game_mod.versions.contains_key(version) {
                let mut existing_version_data = game_mod.versions.get_mut(version).unwrap();

                existing_version_data.download_url = Some(version_info.download_url.clone());
            } else {
                game_mod.versions.insert(
                    version.clone(),
                    GameModVersion {
                        mod_id: mod_id.clone(),
                        file_name: version_info.file_name.clone(),
                        downloaded: false,
                        download_url: Some(version_info.download_url.clone()),
                        metadata: None,
                    },
                );
            }
        }

        match game_mod.selected_version {
            SelectedVersion::Latest(_) => {
                game_mod.selected_version =
                    SelectedVersion::Latest(index_file.latest_version.clone());
            }
            SelectedVersion::LatestIndirect(_) => {
                game_mod.selected_version =
                    SelectedVersion::Latest(index_file.latest_version.clone());
            }
            SelectedVersion::Specific(_) => {}
        }

        game_mod.latest_version = Some(index_file.latest_version.clone());

        debug!("Loaded index file for {}", mod_id);
    }

    warnings
}
