use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use lazy_static::lazy_static;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, IntegrationError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DownloadMode {
    #[serde(rename = "index_file")]
    IndexFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DownloadInfo {
    #[serde(rename = "type")]
    pub download_mode: DownloadMode,
    pub url: String,
}

lazy_static! {
    static ref PARSED_FIELDS: Vec<&'static str> = Vec::from([
        "schema_version",
        "name",
        "mod_id",
        "author",
        "description",
        "mod_version",
        "game_build",
        "sync",
        "homepage",
        "download",
        "integrator"
    ]);
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub schema_version: usize,
    pub name: String,
    pub mod_id: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "version")]
    pub mod_version: String,
    pub game_build: Option<String>,
    pub sync: Option<SyncMode>,
    pub homepage: Option<String>,
    pub download: Option<DownloadInfo>,

    #[serde(default)]
    pub integrator: HashMap<String, Value>,
}

impl Hash for Metadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.schema_version.hash(state);
        self.name.hash(state);
        self.mod_id.hash(state);
        self.author.hash(state);
        self.description.hash(state);
        self.mod_version.hash(state);
        self.game_build.hash(state);
        self.sync.hash(state);
        self.homepage.hash(state);
        self.download.hash(state);

        self.integrator.len().hash(state);
        for (element_name, element) in &self.integrator {
            element_name.hash(state);
            match element {
                Value::String(s) => s.hash(state),
                Value::Number(n) => n.hash(state),
                Value::Bool(b) => b.hash(state),
                Value::Null => (),
                _ => unreachable!(),
            }
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        let cmp = self.schema_version == other.schema_version
            && self.name == other.name
            && self.mod_id == other.mod_id
            && self.author == other.author
            && self.description == other.description
            && self.mod_version == other.mod_version
            && self.game_build == other.game_build
            && self.sync == other.sync
            && self.homepage == other.homepage
            && self.download == other.download
            && self.integrator.len() == other.integrator.len();

        let mut hasher = DefaultHasher::new();
        for (element_name, element) in &self.integrator {
            element_name.hash(&mut hasher);
            match element {
                Value::String(s) => s.hash(&mut hasher),
                Value::Number(n) => n.hash(&mut hasher),
                Value::Bool(b) => b.hash(&mut hasher),
                Value::Null => (),
                _ => unreachable!(),
            }
        }

        let mut other_hasher = DefaultHasher::new();
        for (element_name, element) in &other.integrator {
            element_name.hash(&mut other_hasher);
            match element {
                Value::String(s) => s.hash(&mut other_hasher),
                Value::Number(n) => n.hash(&mut other_hasher),
                Value::Bool(b) => b.hash(&mut other_hasher),
                Value::Null => (),
                _ => unreachable!(),
            }
        }

        cmp & (hasher.finish() == other_hasher.finish())
    }
}

impl Metadata {
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        let value: Value = serde_json::from_slice(slice)?;
        let value = value
            .as_object()
            .ok_or_else(IntegrationError::invalid_metadata)?;

        let schema_version = value["schema_version"].as_u64().unwrap_or(1u64);

        let mut metadata: Metadata = serde_json::from_slice(slice)?;

        match schema_version {
            1 => {
                for (obj_name, value) in value {
                    if !PARSED_FIELDS.contains(&obj_name.as_str()) {
                        metadata.integrator.insert(obj_name.clone(), value.clone());
                    }
                }
            }
            2 => {}
            _ => return Err(IntegrationError::unsupported_schema_version().into()),
        }

        Ok(metadata)
    }
}
