use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

pub struct PerPlatformBoolProperty {
    name: FName,
    property_guid: Guid,
    value: Vec<bool>
}

impl PerPlatformBoolProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let num_entries = cursor.read_i32::<LittleEndian>()?;
        let value = Vec::with_capacity(num_entries);

        for i in 0..num_entries {
            value[i] = cursor.read_bool()?;
        }

        Ok(PerPlatformBoolProperty {
            name,
            property_guid,
            value
        })
    }
}

pub struct PerPlatformIntProperty {
    name: FName,
    property_guid: Guid,
    value: Vec<i32>
}

impl PerPlatformIntProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let num_entries = cursor.read_i32::<LittleEndian>()?;
        let value = Vec::with_capacity(num_entries);

        for i in 0..num_entries {
            value[i] = cursor.read_i32()?;
        }

        Ok(PerPlatformIntProperty {
            name,
            property_guid,
            value
        })
    }
}