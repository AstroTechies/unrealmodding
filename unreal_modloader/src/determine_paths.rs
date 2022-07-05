use std::path::{Path, PathBuf};

use directories::BaseDirs;
use log::{trace, warn};
use steamlocate::SteamDir;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::error::ModLoaderWarning;

#[derive(Debug)]
pub(crate) struct WinStoreInfo {
    pub path: PathBuf,
    pub runtime_id: String,
}

pub(crate) fn determine_base_path_steam(game_name: &str) -> Option<PathBuf> {
    let base_dirs = BaseDirs::new();
    if base_dirs.is_none() {
        warn!("Could not determine base directory");
        return None;
    }
    let base_dirs = base_dirs.unwrap();

    let data_dir = base_dirs.data_local_dir();
    let base_path = Some(data_dir.join(game_name).join("Saved"));
    trace!("base_path: {:?}", base_path);

    base_path
}

pub(crate) fn determine_base_path_winstore(
    store_info: &WinStoreInfo,
    game_name: &str,
) -> Option<PathBuf> {
    let base_dirs = BaseDirs::new();
    if base_dirs.is_none() {
        warn!("Could not determine base directory");
        return None;
    }
    let base_dirs = base_dirs.unwrap();

    let data_dir = base_dirs.data_local_dir();
    let base_path = Some(
        data_dir
            .join("Packages")
            .join(store_info.runtime_id.clone())
            .join("LocalState")
            .join(game_name)
            .join("Saved"),
    );
    trace!("base_path: {:?}", base_path);

    base_path
}

pub(crate) fn determine_install_path_steam(app_id: u32) -> Result<PathBuf, ModLoaderWarning> {
    let steamdir = SteamDir::locate();
    if steamdir.is_none() {
        return Err(ModLoaderWarning::steam_error());
    }

    match steamdir.unwrap().app(&app_id) {
        Some(app) => Ok(app.path.clone()),
        None => Err(ModLoaderWarning::steam_error()),
    }
}

fn convert_runtime_id(package_id: &str) -> Option<String> {
    let id_bits: Vec<&str> = package_id.split("_").collect();

    if id_bits.len() >= 2 {
        return Some(format!("{}_{}", id_bits[0], id_bits[id_bits.len() - 1]));
    }
    None
}

pub(crate) fn determine_install_path_winstore(
    vendor: &str,
) -> Result<WinStoreInfo, ModLoaderWarning> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let packages = hkcu.open_subkey("Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages")
        .or_else(|_| Err(ModLoaderWarning::winstore_error()))?;

    let key_name = packages
        .enum_keys()
        .filter_map(|e| e.ok())
        .filter(|e| e.starts_with(vendor))
        .next()
        .ok_or_else(|| ModLoaderWarning::winstore_error())?;

    let key = packages
        .open_subkey(key_name)
        .or_else(|_| Err(ModLoaderWarning::winstore_error()))?;

    let package_id: String = key
        .get_value("PackageID")
        .or_else(|_| Err(ModLoaderWarning::winstore_error()))?;

    let root_folder: String = key
        .get_value("PackageRootFolder")
        .or_else(|_| Err(ModLoaderWarning::winstore_error()))?;

    let runtime_id: String =
        convert_runtime_id(&package_id).ok_or_else(|| ModLoaderWarning::winstore_error())?;

    Ok(WinStoreInfo {
        path: PathBuf::from(root_folder),
        runtime_id: runtime_id,
    })
}

pub(crate) fn verify_install_path(install_path: &Path, game_name: &str) -> bool {
    let mut exe_name = game_name.to_owned();
    exe_name.push_str(".exe");
    let exe_path = install_path.join(exe_name);
    if !exe_path.is_file() {
        warn!("{}.exe not found in install path", game_name);
        false
    } else {
        true
    }
}
