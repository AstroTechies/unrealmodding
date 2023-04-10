use std::cell::RefCell;
use std::path::PathBuf;

#[cfg(feature = "cpp_loader")]
use std::env;

use crate::config::InstallManager;
use crate::error::ModLoaderWarning;
use crate::game_path_helpers;
use crate::version::GameBuild;

use super::GetGameBuildTrait;

#[derive(Debug)]
pub struct SteamInstallManager {
    pub game_path: RefCell<Option<PathBuf>>,
    pub mods_path: RefCell<Option<PathBuf>>,

    app_id: u32,
    game_name: &'static str,
    game_build_getter: Box<dyn GetGameBuildTrait<SteamInstallManager>>,
}

impl SteamInstallManager {
    pub fn new(
        app_id: u32,
        game_name: &'static str,
        game_build_getter: Box<dyn GetGameBuildTrait<SteamInstallManager>>,
    ) -> Self {
        SteamInstallManager {
            game_path: RefCell::new(None),
            mods_path: RefCell::new(None),

            app_id,
            game_name,
            game_build_getter,
        }
    }
}

impl InstallManager for SteamInstallManager {
    fn get_game_install_path(&self) -> Option<PathBuf> {
        if self.game_path.borrow().is_none() {
            *self.game_path.borrow_mut() =
                game_path_helpers::determine_install_path_steam(self.app_id).ok();
        }
        self.game_path.borrow().clone()
    }

    fn get_paks_path(&self) -> Option<PathBuf> {
        if self.mods_path.borrow().is_none() {
            *self.mods_path.borrow_mut() =
                game_path_helpers::determine_installed_mods_path_steam(self.game_name);
        }
        self.mods_path.borrow().clone()
    }

    fn get_game_build(&self) -> Option<GameBuild> {
        self.game_build_getter.get_game_build(self)
    }

    fn launch_game(&self) -> Result<(), ModLoaderWarning> {
        open::that(format!("steam://run/{}", self.app_id))?;
        Ok(())
    }
}

#[cfg(feature = "cpp_loader")]
impl unreal_cpp_bootstrapper::CppLoaderInstallExtension<ModLoaderWarning> for SteamInstallManager {
    fn get_config_location(&self) -> Result<PathBuf, ModLoaderWarning> {
        Ok(env::temp_dir()
            .join("unrealmodding")
            .join("cpp_loader")
            .join("config.json"))
    }

    fn get_extract_path(&self) -> Result<PathBuf, ModLoaderWarning> {
        Ok(env::temp_dir()
            .join("unrealmodding")
            .join("cpp_loader")
            .join("mods"))
    }

    fn prepare_load(&self) -> Result<(), ModLoaderWarning> {
        let Some(install_path) = self.get_game_install_path() else {
            //todo: error type
            return Ok(());
            // return Err(Box::new(crate::error::ModLoaderWarning::));
        };

        let dest_path = install_path
            .join(self.game_name)
            .join("Binaries")
            .join("Win64");

        super::write_loader_dll(dest_path.as_path())?;
        super::write_proxy_dll(dest_path.as_path())?;

        Ok(())
    }

    // doing nothing on steam as xinput1_3.dll will handle everything
    fn load(&self) -> Result<(), ModLoaderWarning> {
        Ok(())
    }

    fn remove(&self) {
        if let Some(install_path) = self.get_game_install_path() {
            let dest_path = install_path
                .join(self.game_name)
                .join("Binaries")
                .join("Win64");

            super::remove_dlls(dest_path.as_path());
        };
    }
}
