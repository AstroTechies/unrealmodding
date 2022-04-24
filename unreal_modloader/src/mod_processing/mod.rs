use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::debug;

use crate::AppData;
mod index_file;
use index_file::{download_index_files, gather_index_files, insert_index_file_data};
mod pakfile_reading;
use pakfile_reading::{insert_mods_from_readdata, read_pak_files};
mod version_handling;
use version_handling::{auto_pick_versions, set_mod_data_from_version};

mod verify;

pub(crate) fn process_modfiles(
    mod_files: &Vec<PathBuf>,
    data: &Arc<Mutex<AppData>>,
) -> Result<(), Box<dyn Error>> {
    debug!("Processing mod files: {:?}", mod_files);

    // read metadata from pak files and collect for each mod_id
    let mods_read = read_pak_files(mod_files);

    let mut data_guard = data.lock().unwrap();
    let filter = mods_read.keys().cloned().collect();

    // turn metadata into proper data structures
    insert_mods_from_readdata(&mods_read, &mut *data_guard);

    // pick version
    auto_pick_versions(&mut *data_guard);

    // set top level data
    set_mod_data_from_version(&mut *data_guard, &filter);

    // fetch index files
    let index_files_info = gather_index_files(&mut *data_guard, &filter);
    // drop guard to allow UI to render while index files are being downloaded
    drop(data_guard);
    let index_files = download_index_files(index_files_info);
    let mut data_guard = data.lock().unwrap();
    insert_index_file_data(&index_files, &mut *data_guard);

    Ok(())
}
