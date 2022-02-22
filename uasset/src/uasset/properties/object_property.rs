use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Debug)]
pub struct ObjectProperty {
    property_guid: Option<Guid>,
    value: i32
}

#[derive(Debug)]
pub struct AssetObjectProperty {
    property_guid: Option<Guid>,
    value: String
}

#[derive(Debug)]
pub struct SoftObjectProperty {
    property_guid: Option<Guid>,
    value: FName,
    id: u32
}

impl ObjectProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = cursor.read_i32::<LittleEndian>()?;
        Ok(ObjectProperty {
            property_guid,
            value
        })
    }
}
impl AssetObjectProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = cursor.read_string()?;
        Ok(AssetObjectProperty {
            property_guid,
            value
        })
    }
}

impl SoftObjectProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = asset.read_fname()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        Ok(SoftObjectProperty {
            property_guid,
            value,
            id
        })
    }
}