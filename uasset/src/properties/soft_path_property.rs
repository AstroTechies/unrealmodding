use std::io::Cursor;

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    {
        cursor_ext::CursorExt,
        ue4version::VER_UE4_ADDED_SOFT_OBJECT_PATH,
        unreal_types::{FName, Guid},
        Asset,
    },
};

#[derive(Hash, PartialEq, Eq)]
pub struct SoftAssetPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftAssetPathProperty);

#[derive(Hash, PartialEq, Eq)]
pub struct SoftObjectPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftObjectPathProperty);

#[derive(Hash, PartialEq, Eq)]
pub struct SoftClassPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>,
}
impl_property_data_trait!(SoftClassPathProperty);

macro_rules! impl_soft_path_property {
    ($property_name:ident) => {
        impl $property_name {
            pub fn new(
                asset: &mut Asset,
                name: FName,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                let mut path = None;
                let mut asset_path_name = None;
                let mut sub_path = None;

                if asset.engine_version < VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    path = asset.cursor.read_string()?;
                } else {
                    asset_path_name = Some(asset.read_fname()?);
                    sub_path = asset.cursor.read_string()?;
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
            fn write(
                &self,
                asset: &Asset,
                cursor: &mut Cursor<Vec<u8>>,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, cursor, include_header);
                let begin = cursor.position();
                if asset.engine_version < VER_UE4_ADDED_SOFT_OBJECT_PATH {
                    cursor.write_string(&self.path)?;
                } else {
                    asset.write_fname(
                        cursor,
                        self.asset_path_name
                            .as_ref()
                            .ok_or(PropertyError::property_field_none(
                                "asset_path_name",
                                "FName",
                            ))?,
                    )?;
                    cursor.write_string(&self.sub_path)?;
                }

                Ok((cursor.position() - begin) as usize)
            }
        }
    };
}

impl_soft_path_property!(SoftAssetPathProperty);
impl_soft_path_property!(SoftObjectPathProperty);
impl_soft_path_property!(SoftClassPathProperty);
