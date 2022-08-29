use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

use crate::{error, hash_value, Dependency, DownloadInfo, SyncMode};

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = error::Error>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = error::Error>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or version struct")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

fn deserialize_dependency_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Dependency>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "string_or_struct")] Dependency);
    let a: HashMap<String, Wrapper> = HashMap::deserialize(deserializer)?;
    Ok(a.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
}

#[derive(Debug, Default, Clone, Eq, Serialize, Deserialize)]
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

    #[serde(default, deserialize_with = "deserialize_dependency_map")]
    pub dependencies: HashMap<String, Dependency>,

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

        self.dependencies.len().hash(state);
        for (element_name, element) in &self.dependencies {
            element_name.hash(state);
            element.hash(state);
        }

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
            && self.dependencies == other.dependencies
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use semver::VersionReq;

    use crate::{v2::Metadata, DownloadInfo, SyncMode};

    use super::Dependency;

    #[test]
    fn v2_minimal_test() {
        let src = r#"
            {
                "schema_version": 2,
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        let parsed: Metadata = serde_json::from_str(src).unwrap();

        let expected = Metadata {
            schema_version: 2,
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            mod_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(parsed, expected)
    }

    #[test]
    fn v2_simple_test() {
        let src = r#"
            {
                "schema_version": 2,
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
            schema_version: 2,
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
    fn v2_dependency_test() {
        let src = r#"
            {
                "schema_version": 2,
                "name": "Test",
                "mod_id": "TestModId",
                "author": "TestAuthor",
                "description": "Test Description",
                "version": "1.0.0",
                "game_build": "1.2.3",
                "sync": "serverclient",

                "dependencies": {
                    "FirstMod": "*",
                    "SecondMod": ">=1.2.3",
                    "ThirdMod": "<=2.1.0",
                    "FourthMod": {
                        "version": ">=1.2.0",
                        "download": {
                            "type": "index_file",
                            "url": "https://example.com"
                        }
                    }
                }
            }
        "#;

        let parsed: Metadata = serde_json::from_str(src).unwrap();

        let mut dependencies = HashMap::new();
        dependencies.insert(
            "FirstMod".to_string(),
            Dependency::new(VersionReq::parse("*").unwrap(), None),
        );
        dependencies.insert(
            "SecondMod".to_string(),
            Dependency::new(VersionReq::parse(">=1.2.3").unwrap(), None),
        );
        dependencies.insert(
            "ThirdMod".to_string(),
            Dependency::new(VersionReq::parse("<=2.1.0").unwrap(), None),
        );
        dependencies.insert(
            "FourthMod".to_string(),
            Dependency::new(
                VersionReq::parse(">=1.2.0").unwrap(),
                Some(DownloadInfo {
                    download_mode: crate::DownloadMode::IndexFile,
                    url: "https://example.com".to_string(),
                }),
            ),
        );

        let expected = Metadata {
            schema_version: 2,
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            author: Some("TestAuthor".to_string()),
            description: Some("Test Description".to_string()),
            mod_version: "1.0.0".to_string(),
            game_build: Some("1.2.3".to_string()),
            sync: Some(SyncMode::ServerAndClient),
            dependencies,
            ..Default::default()
        };

        assert_eq!(parsed, expected);
    }
}
