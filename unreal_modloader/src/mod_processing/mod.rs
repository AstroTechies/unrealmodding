use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use parking_lot::Mutex;

use crate::error::ModLoaderWarning;
use crate::ModLoaderAppData;
pub(crate) mod dependencies;
pub(crate) mod index_file;
use index_file::{download_index_files, gather_index_files, insert_index_file_data};
mod pakfile_reading;
use pakfile_reading::{insert_mods_from_readdata, read_pak_files};
mod version_handling;
use version_handling::{auto_pick_versions, set_mod_data_from_version};

mod verify;

pub(crate) fn process_modfiles(
    mod_files: &Vec<PathBuf>,
    data: &Arc<Mutex<ModLoaderAppData>>,
    set_enabled: bool,
) -> Vec<ModLoaderWarning> {
    debug!("Processing mod files: {:?}", mod_files);

    let mut warnings = Vec::new();

    // read metadata from pak files and collect for each mod_id
    let (mods_read, read_warnings) = read_pak_files(mod_files);
    warnings.extend(read_warnings);

    let mut data_guard = data.lock();
    let filter: Vec<String> = mods_read.keys().cloned().collect();

    // turn metadata into proper data structures
    insert_mods_from_readdata(&mods_read, &mut data_guard, set_enabled);

    // pick version
    auto_pick_versions(&mut data_guard);

    // set top level data
    set_mod_data_from_version(&mut data_guard, &filter);

    // fetch index files

    // gather index files from all the mods
    let index_files_info = gather_index_files(&mut data_guard, &filter);

    // drop guard to allow UI to render while index files are being downloaded
    drop(data_guard);

    // actually download index files
    let (index_files, index_file_warnings) = download_index_files(index_files_info);
    warnings.extend(index_file_warnings);

    let mut data_guard = data.lock();

    // insert index file data into the mod data
    let insert_warnings = insert_index_file_data(&index_files, &mut data_guard);
    warnings.extend(insert_warnings);

    warnings
}
