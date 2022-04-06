use std::io::{Cursor,};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};
use crate::uasset::error::Error;

macro_rules! parse_int_property {
    ($property_type:ident, $read_func:ident) => {
        pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
            let property_guid = optional_guid!(asset, include_header);

            Ok($property_type {
                name,
                property_guid,
                value: asset.cursor.$read_func::<LittleEndian>()?
            })
        }
    };
}

#[derive(Hash, PartialEq, Eq)]
pub struct Int8Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: i8
}

#[derive(Hash, PartialEq, Eq)]
pub enum ByteType {
    Byte,
    Long
}

#[derive(Hash, PartialEq, Eq)]
pub struct ByteProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub enum_type: Option<i64>,
    pub byte_type: ByteType,
    pub value: i64
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct BoolProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: bool
}

#[derive(Hash, PartialEq, Eq)]
pub struct IntProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: i32
}

#[derive(Hash, PartialEq, Eq)]
pub struct Int16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: i16
}

#[derive(Hash, PartialEq, Eq)]
pub struct Int64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: i64
}

#[derive(Hash, PartialEq, Eq)]
pub struct UInt16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: u16
}

#[derive(Hash, PartialEq, Eq)]
pub struct UInt32Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: u32
}

#[derive(Hash, PartialEq, Eq)]
pub struct UInt64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: u64
}

#[derive(Hash, PartialEq, Eq)]
pub struct FloatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: OrderedFloat<f32>
}

#[derive(Hash, PartialEq, Eq)]
pub struct DoubleProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: OrderedFloat<f64>
}

impl BoolProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let value = asset.cursor.read_bool()?;
        let property_guid = optional_guid!(asset, include_header);

        Ok(BoolProperty {
            name,
            property_guid,
            value
        })
    }
}

impl Int8Property {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        Ok(Int8Property {
            name,
            property_guid,
            value: asset.cursor.read_i8()?
        })
    }
}

impl ByteProperty {
    fn read_byte(asset: &mut Asset, length: i64) -> Result<(ByteType, i64), Error> {
        let value = match length {
            1 => Some((ByteType::Byte, asset.cursor.read_i8()? as i64)),
            0 | 8 => Some((ByteType::Long, asset.cursor.read_i64::<LittleEndian>()?)),
            _ => None
        };

        value.ok_or(Error::invalid_file(format!("Invalid length of {} for ByteProperty", length)))
    }

    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, fallback_length: i64) -> Result<Self, Error> {
        let (property_guid, enum_type) = match include_header {
            true => (asset.read_property_guid()?, Some(asset.cursor.read_i64::<LittleEndian>()?)),
            false => (None, None)
        };

        let (byte_type, value) = ByteProperty::read_byte(asset, length).or(ByteProperty::read_byte(asset, fallback_length))?;

        Ok(ByteProperty {
            name,
            property_guid,
            enum_type,
            byte_type,
            value
        })
    }
}

impl IntProperty {
    parse_int_property!(IntProperty, read_i32);
}

impl Int16Property {
    parse_int_property!(Int16Property, read_i16);
}

impl Int64Property {
    parse_int_property!(Int64Property, read_i64);
}

impl UInt16Property {
    parse_int_property!(UInt16Property, read_u16);
}

impl UInt32Property {
    parse_int_property!(UInt32Property, read_u32);
}

impl UInt64Property {
    parse_int_property!(UInt64Property, read_u64);
}

impl FloatProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(FloatProperty {
            name,
            property_guid,
            value: OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?)
        })
    }
}

impl DoubleProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(DoubleProperty {
            name,
            property_guid,
            value: OrderedFloat(asset.cursor.read_f64::<LittleEndian>()?)
        })
    }
}
