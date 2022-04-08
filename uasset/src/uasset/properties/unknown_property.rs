use std::io::{Cursor, ErrorKind, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write, impl_property_data_trait};
use crate::uasset::properties::{PropertyTrait, PropertyDataTrait};

#[derive(Hash, PartialEq, Eq)]
pub struct UnknownProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<u8>,
}
impl_property_data_trait!(UnknownProperty);

impl UnknownProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, duplication_index: i32) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = Vec::with_capacity(length as usize);
        asset.cursor.read_exact(&mut value);

        Ok(UnknownProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for UnknownProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write(&self.value)?;
        Ok(self.value.len())
    }
}