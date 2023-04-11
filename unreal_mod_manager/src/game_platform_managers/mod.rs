use std::fs;
#[cfg(feature = "cpp_loader")]
use std::io::{Result, Write};
use std::path::Path;

use crate::version::GameBuild;

pub trait GetGameBuildTrait<T>: std::fmt::Debug + Send {
    fn get_game_build(&self, manager: &T) -> Option<GameBuild>;
}

#[cfg(windows)]
mod msstore;
#[cfg(windows)]
pub use msstore::MsStoreInstallManager;

mod proton;
pub use proton::ProtonInstallManager;

#[cfg(windows)]
mod steam;
#[cfg(windows)]
pub use steam::SteamInstallManager;

pub const LOADER_DLL_NAME: &str = "UnrealCppLoader.dll";
#[cfg(feature = "cpp_loader")]
pub fn write_loader_dll(dest_path: &Path) -> Result<()> {
    let loader_dll_path = dest_path.join(LOADER_DLL_NAME);

    let _ = fs::remove_file(&loader_dll_path);

    let mut dll_file = fs::File::create(loader_dll_path)?;
    dll_file.write_all(unreal_cpp_bootstrapper::LOADER_DLL)?;
    dll_file.flush()?;

    Ok(())
}

pub const PROXY_DLL_NAME: &str = "xinput1_3.dll";
#[cfg(feature = "cpp_loader")]
pub fn write_proxy_dll(dest_path: &Path) -> Result<()> {
    let proxy_dll_path = dest_path.join(PROXY_DLL_NAME);

    let _ = fs::remove_file(&proxy_dll_path);

    let mut dll_file = fs::File::create(proxy_dll_path)?;
    dll_file.write_all(unreal_cpp_bootstrapper::PROXY_DLL)?;
    dll_file.flush()?;

    Ok(())
}

pub fn remove_dlls(dest_path: &Path) {
    let loader_dll_path = dest_path.join(LOADER_DLL_NAME);
    let _ = fs::remove_file(loader_dll_path);

    let proxy_dll_path = dest_path.join(PROXY_DLL_NAME);
    let _ = fs::remove_file(proxy_dll_path);
}
