use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct GameplayTagContainerProperty {
    name: FName,
    property_guid: Option<Guid>,
    value: Vec<FName>
}

impl GameplayTagContainerProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let length = cursor.read_i32::<LittleEndian>()?;
        let value = Vec::with_capacity(length as usize);
        for i in 0..length as usize {
            value[i] = asset.read_fname()?;
        }

        Ok(GameplayTagContainerProperty {
            name,
            property_guid,
            value
        })
    }
}