#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use unreal_modintegrator::IntegratorConfig;
use unreal_modloader::config::{GameConfig, InstallManager};
use unreal_modloader::game_platform_managers::{
    GetGameBuildTrait, MsStoreInstallManager, SteamInstallManager,
};
use unreal_modloader::version::GameBuild;

mod astro_integrator;
use astro_integrator::AstroIntegratorConfig;

mod assets;
mod handlers;
mod logging;

use log::info;

#[derive(Debug, Default)]
struct SteamGetGameBuild {
    game_build: RefCell<Option<GameBuild>>,
}

impl GetGameBuildTrait<SteamInstallManager> for SteamGetGameBuild {
    fn get_game_build(&self, manager: &SteamInstallManager) -> Option<GameBuild> {
        if self.game_build.borrow().is_none() && manager.get_game_path().is_some() {
            let version_file_path = manager
                .game_path
                .borrow()
                .as_ref()
                .unwrap()
                .join("build.version");

            if !version_file_path.is_file() {
                info!("{:?} not found", version_file_path);
                return None;
            }

            let version_file = std::fs::read_to_string(&version_file_path).unwrap();
            let game_build_string = version_file.split(' ').next().unwrap().to_owned();

            *self.game_build.borrow_mut() = GameBuild::try_from(&game_build_string).ok();
        }
        self.game_build.borrow().clone()
    }
}

struct AstroGameConfig;

impl<T, E: std::error::Error> GameConfig<'static, AstroIntegratorConfig, T, E> for AstroGameConfig
where
    AstroIntegratorConfig: IntegratorConfig<'static, T, E>,
{
    fn get_integrator_config(&self) -> &astro_integrator::AstroIntegratorConfig {
        &astro_integrator::AstroIntegratorConfig
    }

    fn get_game_build(&self, install_path: &Path) -> Option<GameBuild> {
        let version_file_path = install_path.join("build.version");
        if !version_file_path.is_file() {
            info!("{:?} not found", version_file_path);
            return None;
        }

        let version_file = std::fs::read_to_string(&version_file_path).unwrap();
        let game_build_string = version_file.split(' ').next().unwrap().to_owned();

        GameBuild::try_from(&game_build_string).ok()
    }

    const WINDOW_TITLE: &'static str = "Astroneer Modloader";
    const CONFIG_DIR: &'static str = "AstroModLoader";

    fn get_install_managers(
        &self,
    ) -> std::collections::BTreeMap<&'static str, Box<dyn InstallManager>> {
        let mut managers: std::collections::BTreeMap<&'static str, Box<dyn InstallManager>> =
            BTreeMap::new();

        managers.insert(
            "Steam",
            Box::new(SteamInstallManager::new(
                361420,
                AstroIntegratorConfig::GAME_NAME,
                Box::new(SteamGetGameBuild::default()),
            )),
        );
        managers.insert(
            "Microsoft Store",
            Box::new(MsStoreInstallManager::new(
                "SystemEraSoftworks",
                AstroIntegratorConfig::GAME_NAME,
            )),
        );

        managers
    }
}

fn main() {
    logging::init().unwrap();

    info!("Astroneer Modloader");

    let config = AstroGameConfig;

    unreal_modloader::run(config);
}
