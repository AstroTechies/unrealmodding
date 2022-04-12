use unreal_modintegrator::metadata::{DownloadInfo, SyncMode};

use crate::version::{GameBuild, Version};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedVersion {
    Latest,
    Specific(Version),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMod {
    pub mod_id: String,

    pub versions: Vec<GameModVersion>,
    pub latest_version: Option<GameModVersion>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModVersion {
    pub version: Version,
    pub file_name: String,
    pub downloaded: bool,
}
