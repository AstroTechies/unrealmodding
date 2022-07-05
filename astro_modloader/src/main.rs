#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use unreal_modintegrator::IntegratorConfig;
use unreal_modloader::config::{GameConfig, InstallManager};
use unreal_modloader::game_path_helpers::{self, MsStoreInfo, APPX_MANIFEST_VERSION_REGEX};
use unreal_modloader::version::GameBuild;

mod astro_integrator;
use astro_integrator::AstroIntegratorConfig;

mod assets;
mod handlers;
mod logging;

use log::info;

#[derive(Debug, Default)]
struct SteamInstallManager {
    game_path: RefCell<Option<PathBuf>>,
    mods_path: RefCell<Option<PathBuf>>,
    game_build: RefCell<Option<GameBuild>>,
}

impl InstallManager for SteamInstallManager {
    fn get_game_path(&self) -> Option<PathBuf> {
        if self.game_path.borrow().is_none() {
            *self.game_path.borrow_mut() =
                game_path_helpers::determine_install_path_steam(AstroGameConfig::APP_ID).ok();
        }
        self.game_path.borrow().clone()
    }

    fn get_mods_path(&self) -> Option<PathBuf> {
        if self.mods_path.borrow().is_none() {
            *self.mods_path.borrow_mut() =
                game_path_helpers::determine_base_path_steam(AstroIntegratorConfig::GAME_NAME);
        }
        self.mods_path.borrow().clone()
    }

    fn get_game_build(&self) -> Option<GameBuild> {
        if self.game_build.borrow().is_none() && self.get_game_path().is_some() {
            let version_file_path = self
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

#[derive(Debug, Default)]
struct MsStoreInstallManager {
    store_info: RefCell<Option<MsStoreInfo>>,
    mods_path: RefCell<Option<PathBuf>>,
    game_build: RefCell<Option<GameBuild>>,
}

impl MsStoreInstallManager {
    fn get_store_info(&self) -> Option<MsStoreInfo> {
        if self.store_info.borrow().is_none() {
            *self.store_info.borrow_mut() = game_path_helpers::determine_install_path_winstore(
                AstroGameConfig::WINSTORE_VENDOR_ID.unwrap(),
            )
            .ok()
        }
        self.store_info.borrow().clone()
    }
}

impl InstallManager for MsStoreInstallManager {
    fn get_game_path(&self) -> Option<PathBuf> {
        self.get_store_info().map(|e| e.path).clone()
    }

    fn get_mods_path(&self) -> Option<PathBuf> {
        let store_info = self.get_store_info();
        if self.mods_path.borrow().is_none() && store_info.is_some() {
            *self.mods_path.borrow_mut() = game_path_helpers::determine_base_path_winstore(
                &store_info.unwrap(),
                AstroIntegratorConfig::GAME_NAME,
            );
        }
        self.mods_path.borrow().clone()
    }

    fn get_game_build(&self) -> Option<GameBuild> {
        let game_path = self.get_game_path();
        if self.game_build.borrow().is_none() && game_path.is_some() {
            let mut appx_manifest = File::open(game_path.unwrap().join("AppxManifest.xml")).ok()?;
            let mut manifest_data = String::new();
            appx_manifest.read_to_string(&mut manifest_data).ok()?;

            let matches = APPX_MANIFEST_VERSION_REGEX.captures(&manifest_data);
            let version_match = matches.map(|e| e.get(e.len() - 1)).flatten()?;
            let game_build = GameBuild::try_from(&version_match.as_str().to_string()).ok();
            *self.game_build.borrow_mut() = game_build;
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

    const APP_ID: u32 = 361420;
    const WINSTORE_VENDOR_ID: Option<&'static str> = Some("SystemEraSoftworks");
    const WINDOW_TITLE: &'static str = "Astroneer Modloader";
    const CONFIG_DIR: &'static str = "AstroModLoader";

    fn get_install_managers(
        &self,
    ) -> std::collections::BTreeMap<unreal_modloader::GamePlatform, Box<dyn InstallManager>> {
        let mut managers: std::collections::BTreeMap<
            unreal_modloader::GamePlatform,
            Box<dyn InstallManager>,
        > = BTreeMap::new();

        managers.insert(
            unreal_modloader::GamePlatform::Steam,
            Box::new(SteamInstallManager::default()),
        );
        managers.insert(
            unreal_modloader::GamePlatform::MsStore,
            Box::new(MsStoreInstallManager::default()),
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
