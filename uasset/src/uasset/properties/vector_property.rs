use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::Guid, cursor_ext::CursorExt, types::{Color, Vector, Vector4}}, optional_guid};

#[derive(Debug)]
pub struct VectorProperty {
    property_guid: Option<Guid>,
    value: Vector<f32>
}

#[derive(Debug)]
pub struct IntPointProperty {
    property_guid: Option<Guid>,
    x: i32,
    y: i32
}

#[derive(Debug)]
pub struct Vector4Property {
    property_guid: Option<Guid>,
    value: Vector4<f32>
}

#[derive(Debug)]
pub struct Vector2DProperty {
    property_guid: Option<Guid>,
    x: f32,
    y: f32
}

#[derive(Debug)]
pub struct QuatProperty {
    property_guid: Option<Guid>,
    value: Vector4<f32>
}

#[derive(Debug)]
pub struct RotatorProperty {
    property_guid: Option<Guid>,
    value: Vector<f32>
}

impl VectorProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = Vector::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?
        );
        Ok(VectorProperty {
            property_guid,
            value
        })
    }
}

impl IntPointProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let x = cursor.read_i32::<LittleEndian>()?;
        let y = cursor.read_i32::<LittleEndian>()?;

        Ok(IntPointProperty {
            property_guid,
            x,
            y
        })
    }
}

impl Vector4Property {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let w = cursor.read_f32::<LittleEndian>()?;
        let value = Vector4::new(x, y, z, w);
        Ok(Vector4Property {
            property_guid,
            value
        })
    }
}

impl Vector2DProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;

        Ok(Vector2DProperty {
            property_guid,
            x, y
        })
    }
}

impl QuatProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let w = cursor.read_f32::<LittleEndian>()?;
        let value = Vector4::new(x, y, z, w);

        Ok(QuatProperty {
            property_guid,
            value
        })
    }
}

impl RotatorProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let value = Vector::new(x, y, z);

        Ok(RotatorProperty {
            property_guid,
            value
        })
    }
}