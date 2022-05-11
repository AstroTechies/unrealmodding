use std::io::Cursor;
use std::mem::size_of;

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait,
    {
        unreal_types::{FName, Guid},
        Asset,
    },
};

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct EnumProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub enum_type: Option<FName>,
    pub value: FName,
}
impl_property_data_trait!(EnumProperty);

impl EnumProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };
        let value = asset.read_fname()?;

        Ok(EnumProperty {
            name,
            property_guid,
            duplication_index,
            enum_type,
            value,
        })
    }
}

impl PropertyTrait for EnumProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(
                cursor,
                self.enum_type
                    .as_ref()
                    .ok_or_else(PropertyError::headerless)?,
            )?;
            asset.write_property_guid(cursor, &self.property_guid)?;
        }
        asset.write_fname(cursor, &self.value)?;

        Ok(size_of::<i32>() * 2)
    }
}
