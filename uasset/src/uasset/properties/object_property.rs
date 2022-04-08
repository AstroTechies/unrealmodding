use std::io::{Cursor, ErrorKind};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write};
use crate::uasset::properties::PropertyTrait;

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

impl PropertyTrait for ObjectProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.value)?;
        Ok(size_of::<i32>())
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

impl PropertyTrait for AssetObjectProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_string(&self.value)
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

impl PropertyTrait for SoftObjectProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        asset.write_fname(cursor, &self.value)?;
        cursor.write_u32::<LittleEndian>(self.id)?;
        Ok(size_of::<i32>() * 3)
    }
}
