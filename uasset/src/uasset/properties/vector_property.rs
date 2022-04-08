use std::io::{Cursor, ErrorKind, Read};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ordered_float::OrderedFloat;
use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::{Color, Vector, Vector4}, Asset}, optional_guid, optional_guid_write};
use crate::uasset::properties::PropertyTrait;

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

impl PropertyTrait for VectorProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.value.x.0)?;
        cursor.write_f32::<LittleEndian>(self.value.y.0)?;
        cursor.write_f32::<LittleEndian>(self.value.z.0)?;
        Ok(size_of::<f32>() * 3)
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

impl PropertyTrait for IntPointProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.x)?;
        cursor.write_i32::<LittleEndian>(self.y)?;
        Ok(size_of::<i32>() * 2)
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

impl PropertyTrait for Vector4Property {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.value.x.0)?;
        cursor.write_f32::<LittleEndian>(self.value.y.0)?;
        cursor.write_f32::<LittleEndian>(self.value.z.0)?;
        cursor.write_f32::<LittleEndian>(self.value.w.0)?;
        Ok(size_of::<f32>() * 4)
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

impl PropertyTrait for Vector2DProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.x.0)?;
        cursor.write_f32::<LittleEndian>(self.y.0)?;
        Ok(size_of::<f32>() * 2)
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

impl PropertyTrait for QuatProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.value.x.0)?;
        cursor.write_f32::<LittleEndian>(self.value.y.0)?;
        cursor.write_f32::<LittleEndian>(self.value.z.0)?;
        cursor.write_f32::<LittleEndian>(self.value.w.0)?;
        Ok(size_of::<f32>() * 4)
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

impl PropertyTrait for RotatorProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.value.x.0)?;
        cursor.write_f32::<LittleEndian>(self.value.y.0)?;
        cursor.write_f32::<LittleEndian>(self.value.z.0)?;
        Ok(size_of::<f32>() * 3)
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

impl PropertyTrait for BoxProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        let total_size =
            self.v1.write(asset, cursor, include_header)? +
            self.v2.write(asset, cursor, include_header)?;
        cursor.write_bool(self.is_valid)?;
        Ok(total_size + size_of::<bool>())
    }
}