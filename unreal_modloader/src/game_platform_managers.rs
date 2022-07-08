use std::{cell::RefCell, fmt::Debug, fs::File, io::Read, path::PathBuf};

use crate::{
    config::InstallManager,
    game_path_helpers::{self, MsStoreInfo},
    version::GameBuild,
};

pub trait GetGameBuildTrait<T>: Debug + Send {
    fn get_game_build(&self, manager: &T) -> Option<GameBuild>;
}

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
    fn get_game_path(&self) -> Option<PathBuf> {
        if self.game_path.borrow().is_none() {
            *self.game_path.borrow_mut() =
                game_path_helpers::determine_install_path_steam(self.app_id).ok();
        }
        self.game_path.borrow().clone()
    }

    fn get_mods_path(&self) -> Option<PathBuf> {
        if self.mods_path.borrow().is_none() {
            *self.mods_path.borrow_mut() =
                game_path_helpers::determine_base_path_steam(self.game_name);
        }
        self.mods_path.borrow().clone()
    }

    fn get_game_build(&self) -> Option<GameBuild> {
        self.game_build_getter.get_game_build(self)
    }
}

#[derive(Debug)]
pub struct MsStoreInstallManager {
    store_info: RefCell<Option<MsStoreInfo>>,
    mods_path: RefCell<Option<PathBuf>>,
    game_build: RefCell<Option<GameBuild>>,

    winstore_vendor_id: &'static str,
    game_name: &'static str,
}

impl MsStoreInstallManager {
    pub fn new(winstore_vendor_id: &'static str, game_name: &'static str) -> Self {
        MsStoreInstallManager {
            store_info: RefCell::new(None),
            mods_path: RefCell::new(None),
            game_build: RefCell::new(None),
            winstore_vendor_id,
            game_name,
        }
    }

    fn get_store_info(&self) -> Option<MsStoreInfo> {
        if self.store_info.borrow().is_none() {
            *self.store_info.borrow_mut() =
                game_path_helpers::determine_install_path_winstore(self.winstore_vendor_id).ok()
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
                self.game_name,
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

            let matches = game_path_helpers::APPX_MANIFEST_VERSION_REGEX.captures(&manifest_data);
            let version_match = matches.map(|e| e.get(e.len() - 1)).flatten()?;
            let game_build = GameBuild::try_from(&version_match.as_str().to_string()).ok();
            *self.game_build.borrow_mut() = game_build;
        }

        self.game_build.borrow().clone()
    }
}
