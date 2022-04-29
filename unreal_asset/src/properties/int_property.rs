use std::io::Cursor;
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ordered_float::OrderedFloat;

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write, simple_property_write,
    {
        cursor_ext::CursorExt,
        unreal_types::{FName, Guid},
        Asset,
    },
};

macro_rules! impl_int_property {
    ($property_type:ident, $read_func:ident, $write_func:ident, $ty:ty) => {
        impl $property_type {
            pub fn new(
                asset: &mut Asset,
                name: FName,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                Ok($property_type {
                    name,
                    property_guid,
                    duplication_index,
                    value: asset.cursor.$read_func::<LittleEndian>()?,
                })
            }
        }

        simple_property_write!($property_type, $write_func, value, $ty);
    };
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct Int8Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i8,
}
impl_property_data_trait!(Int8Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub enum ByteType {
    Byte,
    Long,
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct ByteProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub enum_type: Option<i64>,
    pub byte_type: ByteType,
    pub value: i64,
}
impl_property_data_trait!(ByteProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BoolProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: bool,
}
impl_property_data_trait!(BoolProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct IntProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i32,
}
impl_property_data_trait!(IntProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct Int16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i16,
}
impl_property_data_trait!(Int16Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct Int64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i64,
}
impl_property_data_trait!(Int64Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct UInt16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u16,
}
impl_property_data_trait!(UInt16Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct UInt32Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u32,
}
impl_property_data_trait!(UInt32Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct UInt64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u64,
}
impl_property_data_trait!(UInt64Property);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct FloatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: OrderedFloat<f32>,
}
impl_property_data_trait!(FloatProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct DoubleProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: OrderedFloat<f64>,
}
impl_property_data_trait!(DoubleProperty);

impl BoolProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let value = asset.cursor.read_bool()?;
        let property_guid = optional_guid!(asset, include_header);

        Ok(BoolProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for BoolProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        cursor.write_bool(self.value)?;
        optional_guid_write!(self, asset, cursor, include_header);
        Ok(0)
    }
}

impl Int8Property {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        Ok(Int8Property {
            name,
            property_guid,
            duplication_index,
            value: asset.cursor.read_i8()?,
        })
    }
}

impl PropertyTrait for Int8Property {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i8(self.value)?;
        Ok(size_of::<i8>())
    }
}

impl ByteProperty {
    fn read_byte(asset: &mut Asset, length: i64) -> Result<(ByteType, i64), Error> {
        let value = match length {
            1 => Some((ByteType::Byte, asset.cursor.read_i8()? as i64)),
            0 | 8 => Some((ByteType::Long, asset.cursor.read_i64::<LittleEndian>()?)),
            _ => None,
        };

        value.ok_or(Error::invalid_file(format!(
            "Invalid length of {} for ByteProperty",
            length
        )))
    }

    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        length: i64,
        fallback_length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (
                Some(asset.cursor.read_i64::<LittleEndian>()?),
                asset.read_property_guid()?,
            ),
            false => (None, None),
        };

        let (byte_type, value) = ByteProperty::read_byte(asset, length)
            .or_else(|_| ByteProperty::read_byte(asset, fallback_length))?;

        Ok(ByteProperty {
            name,
            property_guid,
            duplication_index,
            enum_type,
            byte_type,
            value,
        })
    }
}

impl PropertyTrait for ByteProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header {
            cursor.write_i64::<LittleEndian>(self.enum_type.ok_or(PropertyError::headerless())?)?;
            asset.write_property_guid(cursor, &self.property_guid)?;
        }

        match self.byte_type {
            ByteType::Byte => {
                cursor.write_u8(self.value as u8)?;
                Ok(1)
            }
            ByteType::Long => {
                cursor.write_i64::<LittleEndian>(self.value)?;
                Ok(8)
            }
        }
    }
}

impl FloatProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(FloatProperty {
            name,
            property_guid,
            duplication_index,
            value: OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
        })
    }
}

impl PropertyTrait for FloatProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.value.0)?;
        Ok(size_of::<f32>())
    }
}

impl DoubleProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(DoubleProperty {
            name,
            property_guid,
            duplication_index,
            value: OrderedFloat(asset.cursor.read_f64::<LittleEndian>()?),
        })
    }
}

impl PropertyTrait for DoubleProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f64::<LittleEndian>(self.value.0)?;
        Ok(size_of::<f64>())
    }
}

impl_int_property!(IntProperty, read_i32, write_i32, i32);
impl_int_property!(Int16Property, read_i16, write_i16, i16);
impl_int_property!(Int64Property, read_i64, write_i64, i64);
impl_int_property!(UInt16Property, read_u16, write_u16, u16);
impl_int_property!(UInt32Property, read_u32, write_u32, u32);
impl_int_property!(UInt64Property, read_u64, write_u64, u64);
