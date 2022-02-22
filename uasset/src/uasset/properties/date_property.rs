use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::Guid, cursor_ext::CursorExt}, optional_guid};

#[derive(Debug)]
pub struct TimeSpanProperty {
    property_guid: Option<Guid>,
    ticks: i64
}

#[derive(Debug)]
pub struct DateTimeProperty {
    property_guid: Option<Guid>,
    ticks: i64
}

impl TimeSpanProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let ticks = cursor.read_i64::<LittleEndian>()?;
        Ok(TimeSpanProperty {
            property_guid,
            ticks
        })
    }
}

impl DateTimeProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let ticks = cursor.read_i64::<LittleEndian>()?;
        Ok(DateTimeProperty {
            property_guid,
            ticks
        })
    }
}
