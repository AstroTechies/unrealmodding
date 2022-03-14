use std::io::{Cursor, Error};

use byteorder::{ReadBytesExt, LittleEndian};

use crate::uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::Vector};

#[derive(Debug)]
pub struct VectorProperty {
    name: FName,
    property_guid: Option<Guid>,
    pos: Vector<f32>
}

impl VectorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => Some(cursor.read_property_guid()?),
            false => None
        };
        
        Ok(VectorProperty {
            name,
            property_guid,
            pos: cursor.read_vector()?
        })
    }
}