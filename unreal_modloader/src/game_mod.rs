use std::collections::HashMap;
use std::fmt;

use unreal_modintegrator::metadata::{DownloadInfo, Metadata, SyncMode};

use crate::version::{GameBuild, Version};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectedVersion {
    /// Used when reading from index file
    Latest(Version),
    /// Used when sorting versions
    LatestIndirect(Option<Version>),
    /// Used when a specific version is selected
    Specific(Version),
}

impl fmt::Display for SelectedVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectedVersion::Latest(version) => write!(f, "Latest ({})", version),
            SelectedVersion::LatestIndirect(version) => {
                if let Some(version) = version {
                    write!(f, "{}", version)
                } else {
                    write!(f, "None")
                }
            }
            SelectedVersion::Specific(version) => write!(f, "{}", version),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameMod {
    pub versions: HashMap<Version, GameModVersion>,
    pub selected_version: SelectedVersion,

    pub active: bool,

    // fields set depending on the selected version
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub game_build: Option<GameBuild>,
    pub sync: SyncMode,
    pub homepage: Option<String>,
    pub download: Option<DownloadInfo>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameModVersion {
    pub file_name: String,
    pub downloaded: bool,
    pub download_url: Option<String>,
    pub metadata: Option<Metadata>,
}
