use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{hash_value, DownloadInfo, SyncMode};

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
            hash_value!(element, state);
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
            hash_value!(element, &mut hasher);
        }

        let mut other_hasher = DefaultHasher::new();
        for (element_name, element) in &other.integrator {
            element_name.hash(&mut other_hasher);
            hash_value!(element, &mut other_hasher);
        }

        cmp && (hasher.finish() == other_hasher.finish())
    }
}
