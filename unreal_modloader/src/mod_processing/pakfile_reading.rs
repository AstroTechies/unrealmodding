use std::collections::HashMap;
use std::fs;

use log::{debug, warn};
use semver::Version;
use unreal_modmetadata::{self, Metadata};
use unreal_pak::PakFile;

use crate::error::ModLoaderWarning;
use crate::game_mod::{GameMod, GameModVersion};
use crate::{FileToProcess, ModLoaderAppData};

use super::verify::{self, MOD_FILENAME_REGEX};

#[derive(Debug)]
pub(crate) struct ReadData(String, Metadata);

pub(crate) fn read_pak_files(
    mod_files: &[FileToProcess],
) -> (HashMap<String, Vec<ReadData>>, Vec<ModLoaderWarning>) {
    let mut mods_read: HashMap<String, Vec<ReadData>> = HashMap::new();
    let mut warnings = Vec::new();

    // read metadata
    for file in mod_files.iter() {
        let file_path = &file.path;
        let file_result = (|| -> Result<(), ModLoaderWarning> {
            let file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned();

            let file = fs::File::open(&file_path)
                .map_err(|err| ModLoaderWarning::from(err).with_mod_id(file_name.clone()))?;
            let mut pak = PakFile::reader(&file);

            pak.load_records()
                .map_err(|err| ModLoaderWarning::from(err).with_mod_id(file_name.clone()))?;

            let record = pak
                .get_record(&String::from("metadata.json"))?
                .data
                .as_ref();
            if record.is_none() {
                return Err(ModLoaderWarning::missing_metadata(file_name));
            }

            let metadata: Metadata =
                unreal_modmetadata::from_slice(record.unwrap()).map_err(|err| {
                    warn!("json error: {}", err);
                    ModLoaderWarning::invalid_metadata(file_name)
                })?;

            let file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned();

            // check that filename generally matches
            if !verify::verify_mod_file_name(&file_name) {
                return Err(ModLoaderWarning::invalid_mod_file_name(file_name));
            }

            let file_name_parts = MOD_FILENAME_REGEX
                .captures(&file_name)
                .ok_or_else(|| ModLoaderWarning::invalid_metadata(file_name.clone()))?;

            let (mod_id, version) = (
                file_name_parts.get(2).unwrap().as_str(),
                file_name_parts.get(3).unwrap().as_str(),
            );

            // check that mod id in file name matches metadata
            if mod_id != metadata.mod_id {
                warn!(
                    "Mod id in file name does not match metadata id: {:?} != {:?}",
                    mod_id, metadata.mod_id
                );
                return Err(ModLoaderWarning::invalid_metadata(file_name));
            }

            // check that version in file name matches metadata
            if version != metadata.mod_version {
                warn!(
                    "Version in file name does not match metadata version: {:?} != {:?}",
                    version, metadata.mod_version
                );
                return Err(ModLoaderWarning::invalid_metadata(file_name));
            }

            let mod_id = metadata.mod_id.to_owned();

            if !mods_read.contains_key(&mod_id) {
                mods_read.insert(mod_id.to_owned(), Vec::new());
            }

            mods_read
                .get_mut(&mod_id)
                .unwrap()
                .push(ReadData(file_name, metadata));

            Ok(())
        })();

        match file_result {
            Ok(_) => {
                debug!(
                    "Successfully read metadata for {:?}",
                    file_path.file_name().unwrap().to_str().unwrap()
                );
            }
            Err(e) => {
                warn!(
                    "Failed to read pak file {:?}, error: {}, deleting...",
                    file_path.file_name().unwrap().to_str().unwrap(),
                    e
                );

                if file.newly_added {
                    let _ = fs::remove_file(file_path);
                }
                warnings.push(e);
            }
        }
    }

    (mods_read, warnings)
}

pub(crate) fn insert_mods_from_readdata(
    mods_read: &HashMap<String, Vec<ReadData>>,
    data: &mut ModLoaderAppData,
    set_enabled: bool,
) {
    for (mod_id, mod_files) in mods_read.iter() {
        // insert metadata
        for read_data in mod_files {
            let version = GameModVersion {
                mod_id: read_data.1.mod_id.clone(),
                file_name: read_data.0.clone(),
                downloaded: true,
                download_url: None,
                metadata: Some(read_data.1.clone()),
            };
            let key: Result<Version, _> =
                Version::parse(&version.metadata.as_ref().unwrap().mod_version);

            data.game_mods
                .entry(mod_id.clone())
                .or_insert_with(|| GameMod {
                    enabled: set_enabled,
                    ..Default::default()
                })
                .versions
                .insert(key.unwrap(), version);
        }
    }
}
