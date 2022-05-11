use std::path::{Path, PathBuf};

use directories::BaseDirs;
use log::{trace, warn};
use steamlocate::SteamDir;

use crate::error::ModLoaderWarning;

pub fn dertermine_base_path(game_name: &str) -> Option<PathBuf> {
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

pub fn dertermine_install_path_steam(app_id: u32) -> Result<PathBuf, ModLoaderWarning> {
    let steamdir = SteamDir::locate();
    if steamdir.is_none() {
        return Err(ModLoaderWarning::steam_error());
    }

    match steamdir.unwrap().app(&app_id) {
        Some(app) => Ok(app.path.clone()),
        None => Err(ModLoaderWarning::steam_error()),
    }
}

pub fn verify_install_path(install_path: &Path, game_name: &str) -> bool {
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
