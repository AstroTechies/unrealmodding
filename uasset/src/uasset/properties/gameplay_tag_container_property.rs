use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct GameplayTagContainerProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<FName>
}

impl GameplayTagContainerProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let length = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(length as usize);
        for i in 0..length as usize {
            value.push(asset.read_fname()?);
        }

        Ok(GameplayTagContainerProperty {
            name,
            property_guid,
            value
        })
    }
}
