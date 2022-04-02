use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct UnknownProperty {
    name: FName,
    property_guid: Option<Guid>,
    value: Vec<u8>
}

impl UnknownProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let mut value = Vec::with_capacity(length as usize);
        cursor.read_exact(&mut value);

        Ok(UnknownProperty {
            name,
            property_guid,
            value
        })
    }
}