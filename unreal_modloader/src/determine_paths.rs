use std::path::PathBuf;

use directories::BaseDirs;
use log::{trace, warn};
use steamlocate::SteamDir;

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

pub fn dertermine_install_path_steam(app_id: u32) -> Option<PathBuf> {
    let mut steamdir = SteamDir::locate().unwrap();
    match steamdir.app(&app_id) {
        Some(app) => Some(app.path.clone()),
        None => None,
    }
}

pub fn verify_install_path(install_path: &PathBuf, game_name: &str) -> bool {
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
