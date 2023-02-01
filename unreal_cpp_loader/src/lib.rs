use std::{error::Error, path::PathBuf};

pub mod config;

pub const LOADER_DLL: &[u8] = include_bytes!(env!("CPP_LOADER_DLL_PATH"));
pub const PROXY_DLL: &[u8] = include_bytes!(env!("CPP_LOADER_PROXY_PATH"));

pub trait CppLoaderInstallExtension<E> {
    fn get_config_location(&self) -> PathBuf;
    fn prepare_load(&self) -> Result<(), E>;
    fn load(&self) -> Result<(), E>;
}
