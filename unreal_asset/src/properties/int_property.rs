use std::io::SeekFrom;
use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::simple_property_write;
use crate::types::{FName, Guid};

macro_rules! impl_int_property {
    ($property_type:ident, $read_func:ident, $write_func:ident, $ty:ty) => {
        impl $property_type {
            pub fn new<Reader: AssetReader>(
                asset: &mut Reader,
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
                    value: asset.$read_func::<LittleEndian>()?,
                })
            }
        }

        simple_property_write!($property_type, $write_func, value, $ty);
    };
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int8Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i8,
}
impl_property_data_trait!(Int8Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum BytePropertyValue {
    Byte(u8),
    FName(FName),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ByteProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub enum_type: Option<FName>,
    pub value: BytePropertyValue,
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

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct IntProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i32,
}
impl_property_data_trait!(IntProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i16,
}
impl_property_data_trait!(Int16Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: i64,
}
impl_property_data_trait!(Int64Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt16Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u16,
}
impl_property_data_trait!(UInt16Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt32Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u32,
}
impl_property_data_trait!(UInt32Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt64Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: u64,
}
impl_property_data_trait!(UInt64Property);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FloatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: OrderedFloat<f32>,
}
impl_property_data_trait!(FloatProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct DoubleProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: OrderedFloat<f64>,
}
impl_property_data_trait!(DoubleProperty);

impl BoolProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let value = asset.read_bool()?;
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
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        asset.write_bool(self.value)?;
        optional_guid_write!(self, asset, include_header);
        Ok(0)
    }
}

impl Int8Property {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
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
            value: asset.read_i8()?,
        })
    }
}

impl PropertyTrait for Int8Property {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i8(self.value)?;
        Ok(size_of::<i8>())
    }
}

impl ByteProperty {
    fn read_value<Reader: AssetReader>(
        asset: &mut Reader,
        length: i64,
    ) -> Result<BytePropertyValue, Error> {
        let value = match length {
            1 => Some(BytePropertyValue::Byte(asset.read_u8()?)),
            8 => Some(BytePropertyValue::FName(asset.read_fname()?)),
            0 => {
                let name_map_pointer = asset.read_i32::<LittleEndian>()?;
                let name_map_index = asset.read_i32::<LittleEndian>()?;

                asset.seek(SeekFrom::Current(-(size_of::<i32>() as i64 * 2)))?;

                let byte_value = if name_map_pointer >= 0
                    && name_map_pointer < asset.get_name_map_index_list().len() as i32
                    && name_map_index == 0
                    && !asset.get_name_reference(name_map_index).contains('/')
                {
                    BytePropertyValue::FName(asset.read_fname()?)
                } else {
                    BytePropertyValue::Byte(asset.read_u8()?)
                };

                Some(byte_value)
            }
            _ => None,
        };

        value.ok_or_else(|| {
            Error::invalid_file(format!("Invalid length of {} for ByteProperty", length))
        })
    }

    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        fallback_length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };

        let value = ByteProperty::read_value(asset, length)
            .or_else(|_| ByteProperty::read_value(asset, fallback_length))?;

        Ok(ByteProperty {
            name,
            property_guid,
            duplication_index,
            enum_type,
            value,
        })
    }
}

impl PropertyTrait for ByteProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(
                self.enum_type
                    .as_ref()
                    .ok_or_else(PropertyError::headerless)?,
            )?;
            asset.write_property_guid(&self.property_guid)?;
        }

        match self.value {
            BytePropertyValue::Byte(value) => {
                asset.write_u8(value)?;
                Ok(1)
            }
            BytePropertyValue::FName(ref name) => {
                asset.write_fname(name)?;
                Ok(8)
            }
        }
    }
}

impl FloatProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
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
            value: OrderedFloat(asset.read_f32::<LittleEndian>()?),
        })
    }
}

impl PropertyTrait for FloatProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.value.0)?;
        Ok(size_of::<f32>())
    }
}

impl DoubleProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
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
            value: OrderedFloat(asset.read_f64::<LittleEndian>()?),
        })
    }
}

impl PropertyTrait for DoubleProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f64::<LittleEndian>(self.value.0)?;
        Ok(size_of::<f64>())
    }
}

impl_int_property!(IntProperty, read_i32, write_i32, i32);
impl_int_property!(Int16Property, read_i16, write_i16, i16);
impl_int_property!(Int64Property, read_i64, write_i64, i64);
impl_int_property!(UInt16Property, read_u16, write_u16, u16);
impl_int_property!(UInt32Property, read_u32, write_u32, u32);
impl_int_property!(UInt64Property, read_u64, write_u64, u64);
