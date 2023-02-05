/// The version 1 of the metadata is considered deprecated.
/// It is recommended to use the version 2.
/// Support for the version 1 will be removed in a future release.
/// The current implementation that ensures backwards compatibility (for now) contains some Astroneer specific data.
use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::Error, hash_value, v2, DownloadInfo, SyncMode};

#[derive(Debug, Default, Clone, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub schema_version: Option<usize>,
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

    /// Astroneer specific data.
    pub persistent_actors: Option<Value>,
    pub mission_trailheads: Option<Value>,
    pub linked_actor_components: Option<Value>,
    pub item_list_entries: Option<Value>,
    pub biome_placement_modifiers: Option<Value>,
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

        if let Some(persistent_actors) = self.persistent_actors.as_ref() {
            hash_value!(persistent_actors, state);
        }
        if let Some(mission_trailheads) = self.mission_trailheads.as_ref() {
            hash_value!(mission_trailheads, state);
        }
        if let Some(linked_actor_components) = self.linked_actor_components.as_ref() {
            hash_value!(linked_actor_components, state);
        }
        if let Some(item_list_entries) = self.item_list_entries.as_ref() {
            hash_value!(item_list_entries, state);
        }
        if let Some(biome_placement_modifiers) = self.biome_placement_modifiers.as_ref() {
            hash_value!(biome_placement_modifiers, state);
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.name == other.name
            && self.mod_id == other.mod_id
            && self.author == other.author
            && self.description == other.description
            && self.mod_version == other.mod_version
            && self.game_build == other.game_build
            && self.sync == other.sync
            && self.homepage == other.homepage
            && self.download == other.download
            && self.persistent_actors == other.persistent_actors
            && self.mission_trailheads == other.mission_trailheads
            && self.linked_actor_components == other.linked_actor_components
            && self.item_list_entries == other.item_list_entries
            && self.biome_placement_modifiers == other.biome_placement_modifiers
    }
}

impl Metadata {
    pub fn to_v2(slice: &[u8]) -> Result<v2::Metadata, Error> {
        let metadata: Metadata = serde_json::from_slice(slice)?;

        let mut integrator = HashMap::new();
        if let Some(persistent_actors) = metadata.persistent_actors {
            integrator.insert("persistent_actors".to_string(), persistent_actors);
        }

        if let Some(mission_trailheads) = metadata.mission_trailheads {
            integrator.insert("mission_trailheads".to_string(), mission_trailheads);
        }

        if let Some(linked_actor_components) = metadata.linked_actor_components {
            integrator.insert(
                "linked_actor_components".to_string(),
                linked_actor_components,
            );
        }

        if let Some(item_list_entries) = metadata.item_list_entries {
            integrator.insert("item_list_entries".to_string(), item_list_entries);
        }

        if let Some(biome_placement_modifiers) = metadata.biome_placement_modifiers {
            integrator.insert(
                "biome_placement_modifiers".to_string(),
                biome_placement_modifiers,
            );
        }

        Ok(v2::Metadata {
            schema_version: 2,
            name: metadata.name,
            mod_id: metadata.mod_id,
            author: metadata.author,
            description: metadata.description,
            mod_version: metadata.mod_version,
            game_build: metadata.game_build,
            sync: metadata.sync,
            homepage: metadata.homepage,
            download: metadata.download,
            integrator,
            dependencies: HashMap::new(),
            cpp_loader_dlls: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{v1::Metadata, SyncMode};

    #[test]
    fn v1_simple_test() {
        let src = r#"
        {
            "schema_version": 1,
            "name": "Test",
            "mod_id": "TestModId",
            "author": "TestAuthor",
            "description": "Test Description",
            "version": "1.0.0",
            "game_build": "1.2.3",
            "sync": "serverclient"
        }
    "#;

        let parsed: Metadata = serde_json::from_str(src).unwrap();

        let expected = Metadata {
            schema_version: Some(1),
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            author: Some("TestAuthor".to_string()),
            description: Some("Test Description".to_string()),
            mod_version: "1.0.0".to_string(),
            game_build: Some("1.2.3".to_string()),
            sync: Some(SyncMode::ServerAndClient),
            ..Default::default()
        };

        assert_eq!(parsed, expected);
    }

    #[test]
    fn v1_minimal_test() {
        let src = r#"
            {
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        let parsed: Metadata = serde_json::from_str(src).unwrap();

        let expected = Metadata {
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            mod_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(parsed, expected)
    }
}
