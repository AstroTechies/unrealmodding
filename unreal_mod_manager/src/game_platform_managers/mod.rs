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
