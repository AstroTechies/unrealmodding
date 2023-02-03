use std::path::PathBuf;

use directories::BaseDirs;
use lazy_static::lazy_static;
use log::{trace, warn};
use regex::Regex;
use steamlocate::SteamDir;

#[cfg(windows)]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::error::ModLoaderWarning;

#[derive(Debug, Clone)]
pub struct MsStoreInfo {
    pub path: PathBuf,
    pub runtime_id: String,
}

lazy_static! {
    pub static ref APPX_MANIFEST_VERSION_REGEX: Regex =
        Regex::new("(?x)<Identity(.*?)Publisher(.*?)Version=\"([^\"]*)\"").unwrap();
}

pub fn determine_installed_mods_path_steam(game_name: &str) -> Option<PathBuf> {
    let base_dirs = BaseDirs::new();
    if base_dirs.is_none() {
        warn!("Could not determine base directory");
        return None;
    }
    let base_dirs = base_dirs.unwrap();

    let data_dir = base_dirs.data_local_dir();
    let base_path = Some(data_dir.join(game_name).join("Saved").join("Paks"));
    trace!("base_path: {:?}", base_path);

    base_path
}

pub fn determine_installed_mods_path_proton(game_name: &str, app_id: u32) -> Option<PathBuf> {
    let data_dir: PathBuf = SteamDir::locate()?
        .path
        .join("steamapps")
        .join("compatdata")
        .join(app_id.to_string())
        .join("pfx")
        .join("drive_c")
        .join("users")
        .join("steamuser")
        .join("AppData")
        .join("Local");
    let base_path = Some(data_dir.join(game_name).join("Saved").join("Paks"));
    trace!("base_path: {:?}", base_path);

    base_path
}

#[cfg(windows)]
pub fn determine_game_package_path_winstore(store_info: &MsStoreInfo) -> Option<PathBuf> {
    let base_dirs = BaseDirs::new();
    let Some(base_dirs) = base_dirs else {

        warn!("Could not determine base directory");
        return None;
    };

    let data_dir = base_dirs.data_local_dir();

    let package_path = Some(
        data_dir
            .join("Packages")
            .join(store_info.runtime_id.clone()),
    );

    trace!("package path: {:?}", package_path);

    package_path
}

#[cfg(windows)]
pub fn determine_installed_mods_path_winstore(
    store_info: &MsStoreInfo,
    game_name: &str,
) -> Option<PathBuf> {
    let package_path = determine_game_package_path_winstore(store_info)?;
    let base_path = Some(
        package_path
            .join("LocalState")
            .join(game_name)
            .join("Saved")
            .join("Paks"),
    );
    trace!("base_path: {:?}", base_path);

    base_path
}

pub fn determine_install_path_steam(app_id: u32) -> Result<PathBuf, ModLoaderWarning> {
    if let Some(mut steam_dir) = SteamDir::locate() {
        match steam_dir.app(&app_id) {
            Some(app) => Ok(app.path.clone()),
            None => Err(ModLoaderWarning::steam_error()),
        }
    } else {
        Err(ModLoaderWarning::steam_error())
    }
}

#[cfg(windows)]
fn convert_runtime_id(package_id: &str) -> Option<String> {
    let id_bits: Vec<&str> = package_id.split('_').collect();

    if id_bits.len() >= 2 {
        return Some(format!("{}_{}", id_bits[0], id_bits[id_bits.len() - 1]));
    }
    None
}

#[cfg(windows)]
pub fn determine_install_path_winstore(vendor: &str) -> Result<MsStoreInfo, ModLoaderWarning> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let packages = hkcu.open_subkey("Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages")
        .map_err(|_| ModLoaderWarning::winstore_error())?;

    let key_name = packages
        .enum_keys()
        .filter_map(|e| e.ok())
        .find(|e| e.starts_with(vendor))
        .ok_or_else(ModLoaderWarning::winstore_error)?;

    let key = packages
        .open_subkey(key_name)
        .map_err(|_| ModLoaderWarning::winstore_error())?;

    let package_id: String = key
        .get_value("PackageID")
        .map_err(|_| ModLoaderWarning::winstore_error())?;

    let root_folder: String = key
        .get_value("PackageRootFolder")
        .map_err(|_| ModLoaderWarning::winstore_error())?;

    let runtime_id: String =
        convert_runtime_id(&package_id).ok_or_else(ModLoaderWarning::winstore_error)?;

    Ok(MsStoreInfo {
        path: PathBuf::from(root_folder),
        runtime_id,
    })
}
