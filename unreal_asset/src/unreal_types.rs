//! Various Unreal Engine types
use std::collections::HashMap;

use crate::error::Error;

pub type Guid = [u8; 16];

/// Create a Guid from 4 u32 values
#[rustfmt::skip]
pub fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    [
        (a & 0xff) as u8, ((a >> 8) & 0xff) as u8, ((a >> 16) & 0xff) as u8, ((a >> 24) & 0xff) as u8,
        (b & 0xff) as u8, ((b >> 8) & 0xff) as u8, ((b >> 16) & 0xff) as u8, ((b >> 24) & 0xff) as u8,
        (c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8, ((c >> 24) & 0xff) as u8,
        (d & 0xff) as u8, ((d >> 8) & 0xff) as u8, ((d >> 16) & 0xff) as u8, ((d >> 24) & 0xff) as u8
    ]
}

/// Create a default Guid filled with all zeroes
pub fn default_guid() -> Guid {
    new_guid(0, 0, 0, 0)
}

#[derive(Debug)]
pub struct GenerationInfo {
    pub export_count: i32,
    pub name_count: i32,
}

impl GenerationInfo {
    pub fn new(export_count: i32, name_count: i32) -> Self {
        GenerationInfo {
            export_count,
            name_count,
        }
    }
}

/// FName is used to store most of the Strings in UE4.
///
/// They are represented by an index+instance number inside a string table inside the asset file.
///
/// Here we are representing them by a string and an instance number.
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct FName {
    pub content: String,
    pub index: i32,
}

pub trait ToFName {
    fn to_fname(&self) -> FName;
}

impl FName {
    pub fn new(content: String, index: i32) -> Self {
        FName { content, index }
    }

    pub fn from_slice(content: &str) -> Self {
        FName {
            content: content.to_string(),
            index: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespacedString {
    pub namespace: Option<String>,
    pub value: Option<String>,
}

impl NamespacedString {
    pub fn new(namespace: Option<String>, value: Option<String>) -> Self {
        NamespacedString { namespace, value }
    }
}

#[derive(Debug, Clone)]
pub struct StringTable {
    pub namespace: Option<String>,
    pub value: HashMap<String, String>,
}

impl StringTable {
    pub fn new(namespace: Option<String>) -> Self {
        StringTable {
            namespace,
            value: HashMap::new(),
        }
    }
}

/// PackageIndex is one of the most important structs in UE4
///
/// It is basically a reference into an import/export table
/// which helps "glue" together assets.
///
/// If a PackageIndex is negative it's an index inside an export table
/// if it's positive it's an index inside an import table.
///
/// When PackageIndex is 0 it makes for a non-existent link.
#[derive(Debug, Hash, Copy, Clone, Default, PartialEq, Eq)]
pub struct PackageIndex {
    pub index: i32,
}

impl PackageIndex {
    pub fn new(index: i32) -> Self {
        PackageIndex { index }
    }

    pub fn is_import(&self) -> bool {
        self.index < 0
    }

    pub fn is_export(&self) -> bool {
        self.index > 0
    }

    pub fn from_import(import_index: i32) -> Result<Self, Error> {
        match import_index < 0 {
            true => Err(Error::invalid_package_index(
                "Import index must be bigger than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(-import_index - 1)),
        }
    }

    pub fn from_export(export_index: i32) -> Result<Self, Error> {
        match export_index < 0 {
            true => Err(Error::invalid_package_index(
                "Export index must be greater than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(export_index + 1)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FieldPath {
    pub path: Vec<FName>,
    pub resolved_owner: PackageIndex,
}

impl FieldPath {
    pub fn new(path: Vec<FName>, resolved_owner: PackageIndex) -> Self {
        FieldPath {
            path,
            resolved_owner,
        }
    }
}
