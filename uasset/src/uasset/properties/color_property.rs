use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::Color}, optional_guid};

#[derive(Debug)]
pub struct ColorProperty {
    name: FName,
    property_guid: Option<Guid>,
    color: Color<u8>
}

#[derive(Debug)]
pub struct LinearColorProperty {
    name: FName,
    property_guid: Option<Guid>,
    color: Color<f32>
}

impl ColorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let color = Color::from_argb(cursor.read_i32::<LittleEndian>()?);
        Ok(ColorProperty {
            name,
            property_guid,
            color
        })
    }
}

impl LinearColorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let color = Color::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?
        );
        Ok(LinearColorProperty {
            name,
            property_guid,
            color
        })
    }
}