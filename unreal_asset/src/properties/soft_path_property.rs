//! Soft path properties

use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersion;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid};

/// Soft asset path property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftAssetPathProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Asset path name
    pub asset_path_name: Option<FName>,
    /// Sub-path
    pub sub_path: Option<String>,
    /// Path
    pub path: Option<String>,
}
impl_property_data_trait!(SoftAssetPathProperty);

/// Soft object path property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftObjectPathProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Asset path name
    pub asset_path_name: Option<FName>,
    /// Sub-path
    pub sub_path: Option<String>,
    /// Path
    pub path: Option<String>,
}
impl_property_data_trait!(SoftObjectPathProperty);

/// Soft class path property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftClassPathProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Asset path name
    pub asset_path_name: Option<FName>,
    /// Sub-path
    pub sub_path: Option<String>,
    /// Path
    pub path: Option<String>,
}
impl_property_data_trait!(SoftClassPathProperty);

/// String asset reference property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct StringAssetReferenceProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Asset path name
    pub asset_path_name: Option<FName>,
    /// Sub-path
    pub sub_path: Option<String>,
    /// Path
    pub path: Option<String>,
}
impl_property_data_trait!(StringAssetReferenceProperty);

macro_rules! impl_soft_path_property {
    ($property_name:ident) => {
        impl $property_name {
            /// Read `$property_name` from an asset
            pub fn new<Reader: AssetReader>(
                asset: &mut Reader,
                name: FName,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                let mut path = None;
                let mut asset_path_name = None;
                let mut sub_path = None;

                if asset.get_object_version() < ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    path = asset.read_fstring()?;
                } else {
                    asset_path_name = Some(asset.read_fname()?);
                    sub_path = asset.read_fstring()?;
                }

                Ok($property_name {
                    name,
                    property_guid,
                    duplication_index,
                    asset_path_name,
                    sub_path,
                    path,
                })
            }
        }

        impl PropertyTrait for $property_name {
            fn write<Writer: AssetWriter>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);
                let begin = asset.position();
                if asset.get_object_version() < ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    asset.write_fstring(self.path.as_deref())?;
                } else {
                    asset.write_fname(self.asset_path_name.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("asset_path_name", "FName")
                    })?)?;
                    asset.write_fstring(self.sub_path.as_deref())?;
                }

                Ok((asset.position() - begin) as usize)
            }
        }
    };
}

impl_soft_path_property!(SoftAssetPathProperty);
impl_soft_path_property!(SoftObjectPathProperty);
impl_soft_path_property!(SoftClassPathProperty);
impl_soft_path_property!(StringAssetReferenceProperty);
