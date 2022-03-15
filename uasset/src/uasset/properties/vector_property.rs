use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::{Color, Vector, Vector4}}, optional_guid};

#[derive(Debug)]
pub struct VectorProperty {
    name: FName,
    property_guid: Option<Guid>,
    value: Vector<f32>
}

#[derive(Debug)]
pub struct IntPointProperty {
    name: FName,
    property_guid: Option<Guid>,
    x: i32,
    y: i32
}

#[derive(Debug)]
pub struct Vector4Property {
    name: FName,
    property_guid: Option<Guid>,
    value: Vector4<f32>
}

#[derive(Debug)]
pub struct Vector2DProperty {
    name: FName,
    property_guid: Option<Guid>,
    x: f32,
    y: f32
}

#[derive(Debug)]
pub struct QuatProperty {
    name: FName,
    property_guid: Option<Guid>,
    value: Vector4<f32>
}

#[derive(Debug)]
pub struct RotatorProperty {
    name: FName,
    property_guid: Option<Guid>,
    value: Vector<f32>
}

impl VectorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = Vector::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?
        );
        Ok(VectorProperty {
            name,
            property_guid,
            value
        })
    }
}

impl IntPointProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let x = cursor.read_i32::<LittleEndian>()?;
        let y = cursor.read_i32::<LittleEndian>()?;

        Ok(IntPointProperty {
            name,
            property_guid,
            x,
            y
        })
    }
}

impl Vector4Property {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let w = cursor.read_f32::<LittleEndian>()?;
        let value = Vector4::new(x, y, z, w);
        Ok(Vector4Property {
            name,
            property_guid,
            value
        })
    }
}

impl Vector2DProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;

        Ok(Vector2DProperty {
            name,
            property_guid,
            x, y
        })
    }
}

impl QuatProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let w = cursor.read_f32::<LittleEndian>()?;
        let value = Vector4::new(x, y, z, w);

        Ok(QuatProperty {
            name,
            property_guid,
            value
        })
    }
}

impl RotatorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let value = Vector::new(x, y, z);

        Ok(RotatorProperty {
            name,
            property_guid,
            value
        })
    }
}

#[derive(Debug)]
pub struct BoxProperty {
    name: FName,
    property_guid: Option<Guid>,
    v1: VectorProperty,
    v2: VectorProperty,
    is_valid: bool
}

impl BoxProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => Some(cursor.read_property_guid()?),
            false => None
        };

        Ok(BoxProperty {
            name,
            property_guid,
            v1: VectorProperty::new(name, cursor, false)?,
            v2: VectorProperty::new(name, cursor, false)?,
            is_valid: cursor.read_bool()?
        })
    }
}