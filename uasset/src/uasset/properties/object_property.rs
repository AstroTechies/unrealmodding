use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: i32
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct AssetObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: String
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SoftObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: FName,
    pub id: u32
}

impl ObjectProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.cursor.read_i32::<LittleEndian>()?;
        Ok(ObjectProperty {
            name,
            property_guid,
            value
        })
    }
}
impl AssetObjectProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.cursor.read_string()?;
        Ok(AssetObjectProperty {
            name,
            property_guid,
            value
        })
    }
}

impl SoftObjectProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fname()?;
        let id = asset.cursor.read_u32::<LittleEndian>()?;
        Ok(SoftObjectProperty {
            name,
            property_guid,
            value,
            id
        })
    }
}
