use error::Error;
use serde::{Deserialize, Serialize};

pub mod error;
pub(crate) mod v1;
pub mod v2;
pub use crate::v2::Metadata;

#[macro_export]
macro_rules! hash_value {
    ($name:expr, $state:expr) => {
        match $name {
            Value::String(s) => s.hash($state),
            Value::Number(n) => n.hash($state),
            Value::Bool(b) => b.hash($state),
            Value::Null => (),
            _ => unreachable!(),
        }
    };
}

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

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::ServerAndClient => write!(f, "Server and Client"),
            SyncMode::ServerOnly => write!(f, "Server only"),
            SyncMode::ClientOnly => write!(f, "Client only"),
            SyncMode::None => write!(f, "None"),
        }
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

pub fn from_slice(slice: &[u8]) -> Result<Metadata, Error> {
    #[derive(Debug, Deserialize)]
    struct VersionMetadata {
        schema_version: Option<u64>,
    }
    let value: VersionMetadata = serde_json::from_slice(slice)?;
    let schema_version = value.schema_version.unwrap_or(1);

    match schema_version {
        1 => Ok(v1::Metadata::to_v2(slice)?),
        2 => Ok(serde_json::from_slice(slice)?),
        _ => Err(Error::unsupported_schema(schema_version)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_slice, Metadata};

    #[test]
    fn v1_no_version_test() {
        let src = r#"
            {
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        let parsed = from_slice(src.as_bytes()).unwrap();

        let expected = Metadata {
            schema_version: 2,
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            mod_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(parsed, expected);
    }

    #[test]
    fn v1_with_version_test() {
        let src = r#"
            {
                "schema_version": 1,
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        let parsed = from_slice(src.as_bytes()).unwrap();

        let expected = Metadata {
            schema_version: 2,
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            mod_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(parsed, expected);
    }

    #[test]
    fn v2_test() {
        let src = r#"
            {
                "schema_version": 2,
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        let parsed = from_slice(src.as_bytes()).unwrap();

        let expected = Metadata {
            schema_version: 2,
            name: "Test".to_string(),
            mod_id: "TestModId".to_string(),
            mod_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(parsed, expected);
    }

    #[test]
    fn unsupported_test() {
        let src = r#"
            {
                "schema_version": 3,
                "name": "Test",
                "mod_id": "TestModId",
                "version": "1.0.0"
            }
        "#;

        assert_eq!(true, from_slice(src.as_bytes()).is_err());
    }
}
