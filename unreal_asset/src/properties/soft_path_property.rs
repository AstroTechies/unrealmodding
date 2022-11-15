use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersion;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid};

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct SoftAssetPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftAssetPathProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SoftObjectPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftObjectPathProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct SoftClassPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftClassPathProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct StringAssetReferenceProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(StringAssetReferenceProperty);

macro_rules! impl_soft_path_property {
    ($property_name:ident) => {
        impl $property_name {
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
                    path = asset.read_string()?;
                } else {
                    asset_path_name = Some(asset.read_fname()?);
                    sub_path = asset.read_string()?;
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
                    asset.write_string(&self.path)?;
                } else {
                    asset.write_fname(self.asset_path_name.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("asset_path_name", "FName")
                    })?)?;
                    asset.write_string(&self.sub_path)?;
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
