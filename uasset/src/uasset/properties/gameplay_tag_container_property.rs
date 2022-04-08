use std::io::{Cursor, ErrorKind};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write, impl_property_data_trait};
use crate::uasset::properties::{PropertyTrait, PropertyDataTrait};

#[derive(Hash, PartialEq, Eq)]
pub struct GameplayTagContainerProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<FName>,
}
impl_property_data_trait!(GameplayTagContainerProperty);


impl GameplayTagContainerProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, duplication_index: i32) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let length = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(length as usize);
        for i in 0..length as usize {
            value.push(asset.read_fname()?);
        }

        Ok(GameplayTagContainerProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for GameplayTagContainerProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.value.len() as i32)?;

        let mut total_size = size_of::<i32>();
        for entry in &self.value {
            asset.write_fname(cursor, entry)?;
            total_size += size_of::<i32>() * 2;
        }

        Ok(total_size)
    }
}