use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    #[serde(rename = "serverclient")]
    ServerAndClient,
    #[serde(rename = "server")]
    ServerOnly,
    #[serde(rename = "client")]
    ClientOnly,
    #[serde(rename = "none")]
    None,
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::ServerAndClient
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadMode {
    #[serde(rename = "index_file")]
    IndexFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadInfo {
    #[serde(rename = "type")]
    pub download_mode: DownloadMode,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub schema_version: i32,
    pub name: String,
    pub mod_id: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "version")]
    pub mod_version: String,
    pub game_build: Option<String>,
    pub sync: SyncMode,
    pub homepage: Option<String>,
    pub download: Option<DownloadInfo>,
}
