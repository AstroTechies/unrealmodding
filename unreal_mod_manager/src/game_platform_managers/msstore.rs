use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use crate::config::InstallManager;
use crate::error::ModLoaderWarning;
use crate::game_path_helpers;
use crate::version::GameBuild;

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
            let mut appx_manifest =
                fs::File::open(game_path.unwrap().join("AppxManifest.xml")).ok()?;
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
        Ok(self.get_loader_dir()?.join("config.json"))
    }

    fn get_extract_path(&self) -> Result<PathBuf, ModLoaderWarning> {
        Ok(self.get_loader_dir()?.join("mods"))
    }

    fn prepare_load(&self) -> Result<(), ModLoaderWarning> {
        super::write_loader_dll(self.get_loader_dir()?.as_path()).map_err(|e| e.into())
    }

    fn load(&self) -> Result<(), ModLoaderWarning> {
        let loader_dir = self.get_loader_dir()?;
        let file_location = loader_dir.join(super::LOADER_DLL_NAME);

        let process = dll_injector::Process::wait_for_process("Astro-UWP64-Shipping")?;

        process.inject_dll(file_location.to_str().unwrap())?;
        Ok(())
    }

    fn remove(&self) {
        // no need to remove any files because there are none written to important dirs.
    }
}
