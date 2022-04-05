use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

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
            true => (Some(asset.read_fname()?), Some(asset.cursor.read_property_guid()?)),
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
