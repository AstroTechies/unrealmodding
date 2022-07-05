use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use unreal_modintegrator::IntegratorConfig;

use crate::{version::GameBuild, GamePlatform};

pub trait InstallManager: Debug + std::marker::Send {
    fn get_game_path(&self) -> Option<PathBuf>;
    fn get_mods_path(&self) -> Option<PathBuf>;
    fn get_game_build(&self) -> Option<GameBuild>;
}

pub trait GameConfig<'a, C, T, E: std::error::Error>: std::marker::Send
where
    C: IntegratorConfig<'a, T, E>,
{
    fn get_integrator_config(&self) -> &C;
    fn get_game_build(&self, install_path: &Path) -> Option<GameBuild>;
    fn get_install_managers(&self) -> BTreeMap<GamePlatform, Box<dyn InstallManager>>;

    const APP_ID: u32;
    const WINSTORE_VENDOR_ID: Option<&'static str>;
    const WINDOW_TITLE: &'static str;
    const CONFIG_DIR: &'static str;
}
