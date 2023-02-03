use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use error::CppBootstrapperError;
use unreal_pak::PakReader;

pub mod config;
pub mod error;

pub const LOADER_DLL: &[u8] = include_bytes!(env!("CPP_LOADER_DLL_PATH"));
pub const PROXY_DLL: &[u8] = include_bytes!(env!("CPP_LOADER_PROXY_PATH"));

pub trait CppLoaderInstallExtension<E> {
    fn get_config_location(&self) -> Result<PathBuf, E>;
    fn get_extract_path(&self) -> Option<PathBuf>;
    fn prepare_load(&self) -> Result<(), E>;
    fn load(&self) -> Result<(), E>;
}

pub fn bootstrap(
    game_name: &str,
    extract_path: &PathBuf,
    path: &PathBuf,
) -> Result<(), CppBootstrapperError> {
    let _ = fs::remove_dir_all(extract_path);
    fs::create_dir_all(extract_path)?;

    let paths = fs::read_dir(path)?;

    for path in paths.filter_map(|e| e.ok()) {
        let file = File::open(path.path())?;
        let mut pak = PakReader::new(&file);
        pak.load_index()?;

        let Ok(metadata_entry) = pak.read_entry(&String::from("metadata.json")) else {
            continue;
        };

        let metadata = unreal_modmetadata::from_slice(&metadata_entry)?;

        for dll in metadata
            .cpp_loader_dlls
            .iter()
            .filter_map(|e| unreal_modmetadata::game_to_absolute(game_name, e))
        {
            let dll_path = PathBuf::from(&dll);
            let dll_data = pak.read_entry(&dll)?;

            let path = extract_path.join(dll_path.file_name().unwrap().to_str().unwrap());

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(path)?;

            file.write_all(&dll_data)?;
        }
    }
    Ok(())
}
