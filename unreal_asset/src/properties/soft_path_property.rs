//! Soft path properties

use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersion;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, Guid};
use crate::unversioned::ancestry::Ancestry;

use super::object_property::SoftObjectPath;

/// Soft path property value
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub enum SoftObjectPathPropertyValue {
    /// asset.get_object_version() < ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH
    Old(Option<String>),
    /// asset.get_object_version() >= ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH
    New(SoftObjectPath),
}

impl SoftObjectPathPropertyValue {
    /// Create a new  `SoftObjectPathPropertyValue` instance
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        match asset.get_object_version() < ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH {
            true => Ok(Self::Old(asset.read_fstring()?)),
            false => Ok(Self::New(SoftObjectPath::new(asset)?)),
        }
    }

    /// Write `SoftObjectPathPropertyValue` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        match self {
            Self::Old(e) => {
                asset.write_fstring(e.as_deref())?;
            }
            Self::New(e) => {
                e.write(asset)?;
            }
        };

        Ok(())
    }
}

/// Soft asset path property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftAssetPathProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: SoftObjectPathPropertyValue,
}
impl_property_data_trait!(SoftAssetPathProperty);

/// Soft object path property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftObjectPathProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: SoftObjectPathPropertyValue,
}
impl_property_data_trait!(SoftObjectPathProperty);

/// Soft class path property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftClassPathProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: SoftObjectPathPropertyValue,
}
impl_property_data_trait!(SoftClassPathProperty);

/// String asset reference property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct StringAssetReferenceProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: SoftObjectPathPropertyValue,
}
impl_property_data_trait!(StringAssetReferenceProperty);

macro_rules! impl_soft_path_property {
    ($property_name:ident) => {
        impl $property_name {
            /// Read `$property_name` from an asset
            pub fn new<Reader: ArchiveReader>(
                asset: &mut Reader,
                name: FName,
                ancestry: Ancestry,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);
                let value = SoftObjectPathPropertyValue::new(asset)?;

                Ok($property_name {
                    name,
                    ancestry,
                    property_guid,
                    duplication_index,
                    value,
                })
            }
        }

        impl PropertyTrait for $property_name {
            fn write<Writer: ArchiveWriter>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);

                let begin = asset.position();

                self.value.write(asset)?;

                Ok((asset.position() - begin) as usize)
            }
        }
    };
}

impl_soft_path_property!(SoftAssetPathProperty);
impl_soft_path_property!(SoftObjectPathProperty);
impl_soft_path_property!(SoftClassPathProperty);
impl_soft_path_property!(StringAssetReferenceProperty);
