use std::collections::HashMap;

use unreal_modintegrator::metadata::{DownloadInfo, SyncMode};

use crate::version::{GameBuild, Version};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectedVersion {
    Latest,
    Specific(Version),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMod {
    //? kinda redundant since it will also be used as the hashmap key
    pub mod_id: String,

    pub versions: HashMap<Version, GameModVersion>,
    pub latest_version: Option<Version>,
    pub selected_version: SelectedVersion,

    pub active: bool,

    // fields set depending on the selected version
    pub name: String,
    pub author: String,
    pub description: String,
    pub game_build: GameBuild,
    pub sync: SyncMode,
    pub homepage: String,
    pub download: Option<DownloadInfo>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameModVersion {
    pub file_name: String,
    pub downloaded: bool,
}
