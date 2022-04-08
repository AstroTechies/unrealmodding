use std::io::{Cursor, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::uasset::error::{Error, PropertyError};
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::VER_UE4_ADDED_SOFT_OBJECT_PATH}, optional_guid, optional_guid_write};
use crate::uasset::properties::PropertyTrait;

#[derive(Hash, PartialEq, Eq)]
pub struct SoftAssetPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>
}

#[derive(Hash, PartialEq, Eq)]
pub struct SoftObjectPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>
}

#[derive(Hash, PartialEq, Eq)]
pub struct SoftClassPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>
}

macro_rules! impl_soft_path_property {
    ($property_name:ident) => {
        impl $property_name {
            pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                let mut path = None;
                let mut asset_path_name = None;
                let mut sub_path = None;

                if asset.engine_version < VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    path = Some(asset.cursor.read_string()?);
                } else {
                    asset_path_name = Some(asset.read_fname()?);
                    sub_path = Some(asset.cursor.read_string()?);
                }

                Ok($property_name {
                    name,
                    property_guid,
                    asset_path_name,
                    sub_path,
                    path
                })
            }
        }

        impl PropertyTrait for $property_name {
            fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
                optional_guid_write!(self, asset, cursor, include_header);
                let begin = cursor.position();
                if asset.engine_version < VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    cursor.write_string(self.path.as_ref().ok_or(PropertyError::property_field_none("path", "String"))?)?;
                } else {
                    asset.write_fname(cursor, self.asset_path_name.as_ref().ok_or(PropertyError::property_field_none("asset_path_name", "FName"))?)?;
                    cursor.write_string(self.sub_path.as_ref().ok_or(PropertyError::property_field_none("sub_path", "String"))?)?;
                }

                Ok((cursor.position() - begin) as usize)
            }
        }
    }
}

impl_soft_path_property!(SoftAssetPathProperty);
impl_soft_path_property!(SoftObjectPathProperty);
impl_soft_path_property!(SoftClassPathProperty);

