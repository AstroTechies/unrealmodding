use std::io::Cursor;
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    {
        cursor_ext::CursorExt,
        unreal_types::{FName, Guid},
        Asset,
    },
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i32,
}
impl_property_data_trait!(ObjectProperty);

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct AssetObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<String>,
}
impl_property_data_trait!(AssetObjectProperty);

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SoftObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: FName,
    pub id: u32,
}
impl_property_data_trait!(SoftObjectProperty);

impl ObjectProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.cursor.read_i32::<LittleEndian>()?;
        Ok(ObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for ObjectProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.value)?;
        Ok(size_of::<i32>())
    }
}

impl AssetObjectProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.cursor.read_string()?;
        Ok(AssetObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for AssetObjectProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_string(&self.value)
    }
}

impl SoftObjectProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fname()?;
        let id = asset.cursor.read_u32::<LittleEndian>()?;
        Ok(SoftObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
            id,
        })
    }
}

impl PropertyTrait for SoftObjectProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        asset.write_fname(cursor, &self.value)?;
        cursor.write_u32::<LittleEndian>(self.id)?;
        Ok(size_of::<i32>() * 3)
    }
}
