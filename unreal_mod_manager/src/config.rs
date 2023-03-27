use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use unreal_mod_integrator::IntegratorConfig;

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

macro_rules! install_manager_trait {
    ($($funcs:item)*) => {
        #[cfg(feature = "cpp_loader")]
        pub trait InstallManager: Debug + std::marker::Send + unreal_cpp_bootstrapper::CppLoaderInstallExtension<ModLoaderWarning> {
            $($funcs)*
        }


        #[cfg(not(feature = "cpp_loader"))]
        pub trait InstallManager: Debug + std::marker::Send {
            $($funcs)*
        }

    };
}

install_manager_trait!(
    fn get_game_install_path(&self) -> Option<PathBuf>;
    fn get_paks_path(&self) -> Option<PathBuf>;
    fn get_game_build(&self) -> Option<GameBuild>;
    fn launch_game(&self) -> Result<(), ModLoaderWarning>;
);

pub trait GameConfig<'data, IC, D, E: std::error::Error + 'static>: std::marker::Send
where
    IC: IntegratorConfig<'data, D, E>,
{
    fn get_integrator_config(&self) -> &IC;
    fn get_game_build(&self, install_path: &Path) -> Option<GameBuild>;
    fn get_install_managers(&self) -> BTreeMap<&'static str, Box<dyn InstallManager>>;

    fn get_newer_update(&self) -> Result<Option<UpdateInfo>, ModLoaderError>;
    fn update_modloader(&self, progress_callback: Box<dyn Fn(f32)>) -> Result<(), ModLoaderError>;

    fn get_icon(&self) -> Option<IconData>;

    #[cfg(feature = "cpp_loader")]
    fn get_cpp_loader_config() -> unreal_cpp_bootstrapper::config::GameSettings;

    const WINDOW_TITLE: &'static str;
    const CONFIG_DIR: &'static str;
    const CRATE_VERSION: &'static str;
}
