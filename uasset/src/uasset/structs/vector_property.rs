use std::io::{Cursor, Error};

use byteorder::{ReadBytesExt, LittleEndian};

use crate::uasset::{unreal_types::Guid, cursor_ext::CursorExt, types::Vector};

#[derive(Debug)]
pub struct VectorProperty {
    property_guid: Option<Guid>,
    pos: Vector<f32>
}

impl VectorProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => Some(cursor.read_property_guid()?),
            false => None
        };
        
        Ok(VectorProperty {
            property_guid,
            pos: cursor.read_vector()?
        })
    }
}