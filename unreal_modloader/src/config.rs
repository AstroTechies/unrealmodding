use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use unreal_modintegrator::IntegratorConfig;

use crate::version::GameBuild;
use crate::{
    error::{ModLoaderError, ModLoaderWarning},
    update_info::UpdateInfo,
};

#[derive(Clone)]
pub struct IconData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub trait InstallManager: Debug + std::marker::Send {
    fn get_game_install_path(&self) -> Option<PathBuf>;
    fn get_paks_path(&self) -> Option<PathBuf>;
    fn get_game_build(&self) -> Option<GameBuild>;
    fn launch_game(&self) -> Result<(), ModLoaderWarning>;
}

pub trait GameConfig<'a, C, T, E: std::error::Error + 'static>: std::marker::Send
where
    C: IntegratorConfig<'a, T, E>,
{
    fn get_integrator_config(&self) -> &C;
    fn get_game_build(&self, install_path: &Path) -> Option<GameBuild>;
    fn get_install_managers(&self) -> BTreeMap<&'static str, Box<dyn InstallManager>>;

    fn get_newer_update(&self) -> Result<Option<UpdateInfo>, ModLoaderError>;
    fn update_modloader(&self, progress_callback: Box<dyn Fn(f32)>) -> Result<(), ModLoaderError>;

    fn get_icon(&self) -> Option<IconData>;

    const WINDOW_TITLE: &'static str;
    const CONFIG_DIR: &'static str;
    const CRATE_VERSION: &'static str;
}
