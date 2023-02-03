use std::{cell::RefCell, fmt::Debug, path::PathBuf};

#[cfg(feature = "cpp_loader")]
use std::{env, fs, io::Write};

#[cfg(windows)]
use std::{fs::File, io::Read};

use crate::{
    config::InstallManager, error::ModLoaderWarning, game_path_helpers, version::GameBuild,
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

    fn get_extract_path(&self) -> Option<PathBuf> {
        Some(
            env::temp_dir()
                .join("unrealmodding")
                .join("cpp_loader")
                .join("mods"),
        )
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

        let proxy_path = dest_path.join("xinput1_3.dll");
        let dll_path = dest_path.join("UnrealCppLoader.dll");

        let _ = fs::remove_file(&proxy_path);
        let _ = fs::remove_file(&dll_path);

        let mut proxy_file = File::create(proxy_path)?;
        proxy_file.write_all(unreal_cpp_bootstrapper::PROXY_DLL)?;

        let mut dll_file = File::create(dll_path)?;
        dll_file.write_all(unreal_cpp_bootstrapper::LOADER_DLL)?;

        Ok(())
    }

    fn load(&self) -> Result<(), ModLoaderWarning> {
        // doing nothing on steam as xinput1_3.dll will handle everything
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProtonInstallManager {
    pub game_path: RefCell<Option<PathBuf>>,
    pub mods_path: RefCell<Option<PathBuf>>,

    app_id: u32,
    game_name: &'static str,
    game_build_getter: Box<dyn GetGameBuildTrait<ProtonInstallManager>>,
}

impl ProtonInstallManager {
    pub fn new(
        app_id: u32,
        game_name: &'static str,
        game_build_getter: Box<dyn GetGameBuildTrait<ProtonInstallManager>>,
    ) -> Self {
        ProtonInstallManager {
            game_path: RefCell::new(None),
            mods_path: RefCell::new(None),

            app_id,
            game_name,
            game_build_getter,
        }
    }
}

impl InstallManager for ProtonInstallManager {
    fn get_game_install_path(&self) -> Option<PathBuf> {
        if self.game_path.borrow().is_none() {
            *self.game_path.borrow_mut() =
                game_path_helpers::determine_install_path_steam(self.app_id).ok();
        }
        self.game_path.borrow().clone()
    }

    fn get_paks_path(&self) -> Option<PathBuf> {
        if self.mods_path.borrow().is_none() {
            *self.mods_path.borrow_mut() = game_path_helpers::determine_installed_mods_path_proton(
                self.game_name,
                self.app_id,
            );
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
impl unreal_cpp_bootstrapper::CppLoaderInstallExtension<ModLoaderWarning> for ProtonInstallManager {
    fn get_config_location(&self) -> Result<PathBuf, ModLoaderWarning> {
        todo!()
    }

    fn get_extract_path(&self) -> Option<PathBuf> {
        todo!()
    }

    fn prepare_load(&self) -> Result<(), ModLoaderWarning> {
        todo!()
    }

    fn load(&self) -> Result<(), ModLoaderWarning> {
        todo!()
    }
}

#[cfg(windows)]
#[derive(Debug)]
pub struct MsStoreInstallManager {
    store_info: RefCell<Option<game_path_helpers::MsStoreInfo>>,
    mods_path: RefCell<Option<PathBuf>>,
    game_build: RefCell<Option<GameBuild>>,

    winstore_vendor_id: &'static str,
    game_name: &'static str,
    state_game_name: &'static str,
}

#[cfg(windows)]
impl MsStoreInstallManager {
    pub fn new(
        winstore_vendor_id: &'static str,
        game_name: &'static str,
        state_game_name: &'static str,
    ) -> Self {
        MsStoreInstallManager {
            store_info: RefCell::new(None),
            mods_path: RefCell::new(None),
            game_build: RefCell::new(None),
            winstore_vendor_id,
            game_name,
            state_game_name,
        }
    }

    fn get_store_info(&self) -> Option<game_path_helpers::MsStoreInfo> {
        if self.store_info.borrow().is_none() {
            *self.store_info.borrow_mut() =
                game_path_helpers::determine_install_path_winstore(self.winstore_vendor_id).ok()
        }
        self.store_info.borrow().clone()
    }

    #[cfg(feature = "cpp_loader")]
    fn get_loader_dir(&self) -> Result<PathBuf, ModLoaderWarning> {
        let Some(store_info) = self.get_store_info() else {
            return Err(ModLoaderWarning::winstore_error());
        };

        let Some(package_path) = game_path_helpers::determine_game_package_path_winstore(
            &store_info
        ) else {
            return Err(ModLoaderWarning::winstore_error());
        };

        let tmp_dir = package_path.join("AC").join("Temp");
        let loader_dir = tmp_dir.join("unrealmodding").join("cpp_loader");

        fs::create_dir_all(&loader_dir)?;

        Ok(loader_dir)
    }
}

#[cfg(windows)]
impl InstallManager for MsStoreInstallManager {
    fn get_game_install_path(&self) -> Option<PathBuf> {
        self.get_store_info().map(|e| e.path)
    }

    fn get_paks_path(&self) -> Option<PathBuf> {
        if let Some(store_info) = self.get_store_info() {
            if self.mods_path.borrow().is_none() {
                *self.mods_path.borrow_mut() =
                    game_path_helpers::determine_installed_mods_path_winstore(
                        &store_info,
                        self.state_game_name,
                    );
            }
        }
        self.mods_path.borrow().clone()
    }

    fn get_game_build(&self) -> Option<GameBuild> {
        let game_path = self.get_game_install_path();
        if self.game_build.borrow().is_none() && game_path.is_some() {
            let mut appx_manifest = File::open(game_path.unwrap().join("AppxManifest.xml")).ok()?;
            let mut manifest_data = String::new();
            appx_manifest.read_to_string(&mut manifest_data).ok()?;

            let matches = game_path_helpers::APPX_MANIFEST_VERSION_REGEX.captures(&manifest_data);
            let version_match = matches.and_then(|e| e.get(e.len() - 1))?;
            let game_build = GameBuild::try_from(&version_match.as_str().to_string()).ok();
            *self.game_build.borrow_mut() = game_build;
        }

        *self.game_build.borrow()
    }

    fn launch_game(&self) -> Result<(), ModLoaderWarning> {
        let store_info = self.get_store_info();
        if let Some(store_info) = store_info {
            open::that(format!(
                "shell:appsFolder\\{}!{}",
                store_info.runtime_id, self.game_name
            ))?;
        } else {
            return Err(ModLoaderWarning::winstore_error());
        }

        Ok(())
    }
}

#[cfg(windows)]
#[cfg(feature = "cpp_loader")]
impl unreal_cpp_bootstrapper::CppLoaderInstallExtension<ModLoaderWarning>
    for MsStoreInstallManager
{
    fn get_config_location(&self) -> Result<PathBuf, ModLoaderWarning> {
        self.get_loader_dir().map(|e| e.join("config.json"))
    }

    fn get_extract_path(&self) -> Option<PathBuf> {
        self.get_loader_dir().ok().map(|e| e.join("mods"))
    }

    fn prepare_load(&self) -> Result<(), ModLoaderWarning> {
        let loader_dir = self.get_loader_dir()?;
        let file_location = loader_dir.join("loader.dll");

        let _ = fs::remove_file(&file_location);

        let mut dll_file = File::create(file_location)?;
        dll_file.write_all(unreal_cpp_bootstrapper::LOADER_DLL)?;
        dll_file.flush()?;
        Ok(())
    }

    fn load(&self) -> Result<(), ModLoaderWarning> {
        let loader_dir = self.get_loader_dir()?;
        let file_location = loader_dir.join("loader.dll");

        let process = dll_injector::Process::wait_for_process("Astro-UWP64-Shipping")?;

        process.inject_dll(file_location.to_str().unwrap())?;
        Ok(())
    }
}
