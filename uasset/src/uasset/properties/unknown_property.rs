use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct UnknownProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<u8>
}

impl UnknownProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = Vec::with_capacity(length as usize);
        asset.cursor.read_exact(&mut value);

        Ok(UnknownProperty {
            name,
            property_guid,
            value
        })
    }
}
