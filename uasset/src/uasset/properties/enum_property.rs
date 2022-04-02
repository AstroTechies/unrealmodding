use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct EnumProperty {
    name: FName,
    property_guid: Option<Guid>,
    enum_type: Option<FName>,
    value: FName
}

impl EnumProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, asset: &Asset) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), Some(cursor.read_property_guid()?)),
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