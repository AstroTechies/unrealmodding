use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TimeSpanProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub ticks: i64
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DateTimeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub ticks: i64
}

impl TimeSpanProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let ticks = cursor.read_i64::<LittleEndian>()?;
        Ok(TimeSpanProperty {
            name,
            property_guid,
            ticks
        })
    }
}

impl DateTimeProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let ticks = cursor.read_i64::<LittleEndian>()?;
        Ok(DateTimeProperty {
            name,
            property_guid,
            ticks
        })
    }
}
