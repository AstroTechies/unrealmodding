use std::io::{Cursor, ErrorKind, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write};
use crate::uasset::properties::PropertyTrait;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct GuidProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Guid
}

impl GuidProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = [0u8; 16];
        asset.cursor.read_exact(&mut value)?;
        Ok(GuidProperty {
            name,
            property_guid,
            value
        })
    }
}

impl PropertyTrait for GuidProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(asset, cursor, include_header);
        cursor.write(&self.value)?;
        Ok(16)
    }
}
