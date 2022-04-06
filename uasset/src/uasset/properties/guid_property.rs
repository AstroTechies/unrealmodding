use std::io::{Cursor, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

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
