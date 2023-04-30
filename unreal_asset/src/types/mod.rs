//! Unreal types

pub mod movie;
pub mod package_object_index;
pub mod vector;

use crate::{asset::name_map::NameMap, containers::shared_resource::SharedResource, error::Error};
use std::hash::Hash;

/// FName is used to store most of the Strings in UE4.
///
/// They are represented by an index+instance number inside a string table inside the asset file.
///
/// Here we are representing them by a string and an instance number.
#[derive(Debug, Clone)]
pub enum FName {
    /// Backed FName that is part of a namemap
    Backed {
        /// FName name map index
        index: i32,
        /// FName instance number
        number: i32,
        /// Namemap which this FName belongs to
        name_map: SharedResource<NameMap>,
    },
    /// Dummy FName that is not backed by any namemap, trying to serialize this will result in an `FNameError`
    Dummy {
        /// FName value
        value: String,
        /// FName instance number
        number: i32,
    },
}

/// Get implementer serialized name
pub trait ToSerializedName {
    /// Convert to serialized name
    ///
    /// # Warning
    ///
    /// This function is dangerous to call when a mutable reference to a name map exists
    /// Doing so may result in a panic
    fn to_serialized_name(&self) -> String;
}

impl FName {
    /// Create a new `FName` instance with an index
    pub fn new(index: i32, number: i32, name_map: SharedResource<NameMap>) -> Self {
        FName::Backed {
            index,
            number,
            name_map,
        }
    }

    /// Create a new "dummy" `FName` instance from a slice and an index
    pub fn new_dummy(value: String, number: i32) -> Self {
        FName::Dummy { value, number }
    }

    /// Create a new "dummy" `FName` instance from a slice with an index of 0
    pub fn from_slice(value: &str) -> Self {
        FName::new_dummy(value.to_string(), 0)
    }

    /// Get this FName content
    pub fn get_content(&self) -> String {
        // todo: return string ref
        match self {
            FName::Backed {
                index,
                number: _,
                name_map,
            } => name_map.get_ref().get_name_reference(*index),
            FName::Dummy { value, number: _ } => value.clone(),
        }
    }
}

impl PartialEq for FName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                FName::Backed {
                    index: a_index,
                    number: a_number,
                    name_map: _,
                },
                FName::Backed {
                    index: b_index,
                    number: b_number,
                    name_map: _,
                },
            ) => a_index == b_index && a_number == b_number,
            (
                FName::Dummy {
                    value: a_value,
                    number: a_number,
                },
                FName::Dummy {
                    value: b_value,
                    number: b_number,
                },
            ) => a_value == b_value && a_number == b_number,
            _ => false,
        }
    }
}

impl Eq for FName {}

impl Hash for FName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            FName::Backed {
                index,
                number,
                name_map: _,
            } => {
                index.hash(state);
                number.hash(state);
            }
            FName::Dummy { value, number } => {
                value.hash(state);
                number.hash(state);
            }
        }
    }
}

impl Default for FName {
    fn default() -> Self {
        FName::Dummy {
            value: String::default(),
            number: i32::default(),
        }
    }
}

/// PackageIndex is one of the most important structs in UE4
///
/// It is basically a reference into an import/export table
/// which helps "glue" together assets.
///
/// If a PackageIndex is negative it's an index inside an import table
/// if it's positive it's an index inside an export table.
///
/// When PackageIndex is 0 it makes for a non-existent link.
#[derive(Debug, Hash, Copy, Clone, Default, PartialEq, Eq)]
pub struct PackageIndex {
    /// Index
    pub index: i32,
}

impl PackageIndex {
    /// Create a new `PackageIndex`
    pub fn new(index: i32) -> Self {
        PackageIndex { index }
    }

    /// Check if this index is an import
    pub fn is_import(&self) -> bool {
        self.index < 0
    }

    /// Check if this index is an export
    pub fn is_export(&self) -> bool {
        self.index > 0
    }

    /// Create a `PackageIndex` from an import index
    pub fn from_import(import_index: i32) -> Result<Self, Error> {
        match import_index < 0 {
            true => Err(Error::invalid_package_index(
                "Import index must be bigger than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(-import_index - 1)),
        }
    }

    /// Create a `PackageIndex` from an export index
    pub fn from_export(export_index: i32) -> Result<Self, Error> {
        match export_index < 0 {
            true => Err(Error::invalid_package_index(
                "Export index must be greater than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(export_index + 1)),
        }
    }
}

/// Guid
pub type Guid = [u8; 16];

/// Create a Guid from 4 u32 values
#[rustfmt::skip]
pub const fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    [
        (a & 0xff) as u8, ((a >> 8) & 0xff) as u8, ((a >> 16) & 0xff) as u8, ((a >> 24) & 0xff) as u8,
        (b & 0xff) as u8, ((b >> 8) & 0xff) as u8, ((b >> 16) & 0xff) as u8, ((b >> 24) & 0xff) as u8,
        (c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8, ((c >> 24) & 0xff) as u8,
        (d & 0xff) as u8, ((d >> 8) & 0xff) as u8, ((d >> 16) & 0xff) as u8, ((d >> 24) & 0xff) as u8
    ]
}

/// Create a default Guid filled with all zeros
pub fn default_guid() -> Guid {
    new_guid(0, 0, 0, 0)
}

/// Asset generation info
#[derive(Debug)]
pub struct GenerationInfo {
    /// Export count
    pub export_count: i32,
    /// Name count
    pub name_count: i32,
}

impl GenerationInfo {
    /// Create a new `GenerationInfo` instance
    pub fn new(export_count: i32, name_count: i32) -> Self {
        GenerationInfo {
            export_count,
            name_count,
        }
    }
}
