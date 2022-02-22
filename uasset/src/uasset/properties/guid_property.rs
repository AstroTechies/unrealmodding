use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::Guid, cursor_ext::CursorExt}, optional_guid};

#[derive(Debug)]
pub struct GuidProperty {
    property_guid: Option<Guid>,
    value: Guid
}

impl GuidProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let mut value = [0u8; 16];
        cursor.read_exact(&mut value)?;
        Ok(GuidProperty {
            property_guid,
            value
        })
    }
}