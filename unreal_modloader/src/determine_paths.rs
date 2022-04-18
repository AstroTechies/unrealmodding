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

pub fn dertermine_install_path(app_id: u32) -> Option<PathBuf> {
    let mut steamdir = SteamDir::locate().unwrap();
    match steamdir.app(&app_id) {
        Some(app) => Some(app.path.clone()),
        None => None,
    }
}
