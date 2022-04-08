use std::io::{Cursor, ErrorKind};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::uasset::error::{Error, PropertyError};
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};
use crate::uasset::properties::PropertyTrait;

#[derive(Hash, PartialEq, Eq)]
pub struct EnumProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub enum_type: Option<FName>,
    pub value: FName
}

impl EnumProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None)
        };
        let value = asset.read_fname()?;

        Ok(EnumProperty {
            name,
            property_guid,
            enum_type,
            value
        })
    }
}

impl PropertyTrait for EnumProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(cursor, self.enum_type.as_ref().ok_or(PropertyError::headerless())?)?;
            asset.write_property_guid(cursor, &self.property_guid)?;
        }
        asset.write_fname(cursor, &self.value)?;

        Ok(size_of::<i32>() * 2)
    }
}