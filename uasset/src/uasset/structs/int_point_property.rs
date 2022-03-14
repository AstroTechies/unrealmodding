use std::io::{Cursor, Error};
use byteorder::{ReadBytesExt, LittleEndian};

use crate::uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt};


#[derive(Debug)]
pub struct IntPointProperty {
    name: FName,
    property_guid: Option<Guid>,
    x: i32,
    y: i32
}

impl IntPointProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => Some(cursor.read_property_guid()?),
            false => None
        };

        Ok(IntPointProperty {
            name,
            property_guid,
            x: cursor.read_i32::<LittleEndian>()?,
            y: cursor.read_i32::<LittleEndian>()?
        })
    }
}
