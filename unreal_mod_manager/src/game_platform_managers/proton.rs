use std::cell::RefCell;
use std::path::PathBuf;

#[cfg(feature = "cpp_loader")]
use std::io::Write;

use crate::config::InstallManager;
use crate::error::ModLoaderWarning;
use crate::game_path_helpers;
use crate::version::GameBuild;

use super::GetGameBuildTrait;

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
        game_path_helpers::determine_user_path_proton(self.app_id)
            .map(|e| {
                e.join("Temp")
                    .join("unrealmodding")
                    .join("cpp_loader")
                    .join("config.json")
            })
            .ok_or_else(ModLoaderWarning::steam_error)
    }

    fn get_extract_path(&self) -> Result<PathBuf, ModLoaderWarning> {
        Ok(game_path_helpers::determine_user_path_proton(self.app_id)
            .ok_or(ModLoaderWarning::other(String::from(
                "Failed to find proton user path!",
            )))?
            .join("Temp")
            .join("unrealmodding")
            .join("cpp_loader")
            .join("mods"))
    }

    fn prepare_load(&self) -> Result<(), ModLoaderWarning> {
        let Some(install_path) = self.get_game_install_path() else {
            return Err(ModLoaderWarning::steam_error());
        };

        let Some(prefix_path) = game_path_helpers::determine_prefix_path_proton(self.app_id) else {
            return Err(ModLoaderWarning::steam_error());
        };

        let registry_path = install_path.join("reg.reg");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(registry_path)?;

        let mut writer = std::io::BufWriter::new(file);
        write!(writer, "Windows Registry Editor Version 5.00")?;

        write!(
            writer,
            "[HKEY_CURRENT_USER\\Software\\Wine\\AppDefaults\\{}-Win64-Shipping.exe\\DllOverrides]",
            self.game_name
        )?;

        write!(writer, "\"xinput1_3\"=\"native,builtin\"")?;

        drop(writer);

        let _ = std::process::Command::new("wine")
            .args(["regedit", "C:\\Users\\steamuser\\reg.reg"])
            .env("WINEPREFIX", prefix_path.to_str().unwrap())
            .output()?;

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
