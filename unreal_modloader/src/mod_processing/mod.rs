use crate::AppData;
use std::error::Error;
use std::fs;
use std::sync::{Arc, Mutex};

mod index_file;
use index_file::{download_index_files, gather_index_files};
mod pakfile_reading;
use pakfile_reading::{insert_mods_from_readdata, read_pak_files};
mod version_handling;
use version_handling::{auto_pick_versions, set_mod_data_from_version};

pub(crate) fn process_modfiles(
    mod_files: &Vec<fs::DirEntry>,
    data: &Arc<Mutex<AppData>>,
) -> Result<(), Box<dyn Error>> {
    // read metadata from pak files and collect for each mod_id
    let mods_read = read_pak_files(mod_files);

    let mut data_guard = data.lock().unwrap();

    // turn metadata into proper data structures
    insert_mods_from_readdata(&mods_read, &mut *data_guard);

    // pick version
    auto_pick_versions(&mut *data_guard);

    // set top level data
    set_mod_data_from_version(&mut *data_guard);

    // fetch index files
    let index_files = gather_index_files(&mut *data_guard);
    println!("Index files: {:#?}", index_files);
    drop(data_guard);

    download_index_files(index_files);

    Ok(())
}
