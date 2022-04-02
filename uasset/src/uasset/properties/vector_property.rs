use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::{Color, Vector, Vector4}}, optional_guid};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct VectorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vector<OrderedFloat<f32>>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct IntPointProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub x: i32,
    pub y: i32
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vector4Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vector4<OrderedFloat<f32>>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vector2DProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct QuatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vector4<OrderedFloat<f32>>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct RotatorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vector<OrderedFloat<f32>>
}

impl VectorProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = Vector::new(
            OrderedFloat(cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(cursor.read_f32::<LittleEndian>()?)
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
        
        let x = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
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
        
        let x = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(cursor.read_f32::<LittleEndian>()?);

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
        
        let x = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
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
        
        let x = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let value = Vector::new(x, y, z);

        Ok(RotatorProperty {
            name,
            property_guid,
            value
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct BoxProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub v1: VectorProperty,
    pub v2: VectorProperty,
    pub is_valid: bool
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
