use std::collections::{BTreeMap, HashMap};
use std::fmt;

use semver::Version;
use unreal_mod_metadata::{Dependency, DownloadInfo, Metadata, SyncMode};

use crate::version::GameBuild;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectedVersion {
    /// Used when reading from index file
    Latest(Version),
    /// Used when sorting versions
    LatestIndirect(Option<Version>),
    /// Used when a specific version is selected
    Specific(Version),
}

impl Default for SelectedVersion {
    fn default() -> Self {
        Self::LatestIndirect(None)
    }
}

impl SelectedVersion {
    pub fn unwrap(self) -> Version {
        match self {
            SelectedVersion::Latest(version) => version,
            SelectedVersion::LatestIndirect(version) => version.unwrap(),
            SelectedVersion::Specific(version) => version,
        }
    }

    pub fn is_latest(&self) -> bool {
        match self {
            SelectedVersion::Latest(_) => true,
            SelectedVersion::LatestIndirect(_) => true,
            SelectedVersion::Specific(_) => false,
        }
    }
}

impl fmt::Display for SelectedVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectedVersion::Latest(version) => write!(f, "Latest ({version})"),
            SelectedVersion::LatestIndirect(version) => {
                if let Some(version) = version {
                    write!(f, "{version}*")
                } else {
                    write!(f, "None")
                }
            }
            SelectedVersion::Specific(version) => write!(f, "{version}"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameMod {
    pub versions: BTreeMap<Version, GameModVersion>,
    pub selected_version: SelectedVersion,
    pub latest_version: Option<Version>,

    pub enabled: bool,

    // fields set depending on the selected version
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub game_build: Option<GameBuild>,
    pub sync: SyncMode,
    pub homepage: Option<String>,
    pub download: Option<DownloadInfo>,
    pub dependencies: HashMap<String, Dependency>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameModVersion {
    pub mod_id: String,
    pub file_name: String,
    pub downloaded: bool,
    pub download_url: Option<String>,
    pub metadata: Option<Metadata>,
}
