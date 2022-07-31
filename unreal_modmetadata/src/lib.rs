use error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    let value: Value = serde_json::from_slice(slice)?;
    let value = value.as_object().ok_or_else(Error::invalid_metadata)?;
    let schema_version = value["schema_version"].as_u64().unwrap_or(1u64);

    match schema_version {
        1 => Ok(v1::Metadata::to_v2(slice)?),
        2 => Ok(serde_json::from_slice(slice)?),
        _ => Err(Error::unsupported_schema(schema_version)),
    }
}
