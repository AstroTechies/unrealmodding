use std::io::{Cursor, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;
use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::{Color, Vector, Vector4}, Asset}, optional_guid};

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
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = Vector::new(
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?)
        );
        Ok(VectorProperty {
            name,
            property_guid,
            value
        })
    }
}

impl IntPointProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let x = asset.cursor.read_i32::<LittleEndian>()?;
        let y = asset.cursor.read_i32::<LittleEndian>()?;

        Ok(IntPointProperty {
            name,
            property_guid,
            x,
            y
        })
    }
}

impl Vector4Property {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let x = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let value = Vector4::new(x, y, z, w);
        Ok(Vector4Property {
            name,
            property_guid,
            value
        })
    }
}

impl Vector2DProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let x = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

        Ok(Vector2DProperty {
            name,
            property_guid,
            x, y
        })
    }
}

impl QuatProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let x = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let value = Vector4::new(x, y, z, w);

        Ok(QuatProperty {
            name,
            property_guid,
            value
        })
    }
}

impl RotatorProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let x = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
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
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => asset.read_property_guid()?,
            false => None
        };

        Ok(BoxProperty {
            name: name.clone(),
            property_guid,
            v1: VectorProperty::new(asset, name.clone(), false)?,
            v2: VectorProperty::new(asset, name.clone(), false)?,
            is_valid: asset.cursor.read_bool()?
        })
    }
}
