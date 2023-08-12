//! Int properties

use crate::property_prelude::*;

/// Implement a simple integer property
macro_rules! impl_int_property {
    ($property_type:ident, $read_func:ident, $write_func:ident, $ty:ty) => {
        impl $property_type {
            /// Read `$property_type` from an asset
            pub fn new<Reader: ArchiveReader>(
                asset: &mut Reader,
                name: FName,
                ancestry: Ancestry,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                Ok($property_type {
                    name,
                    ancestry,
                    property_guid,
                    duplication_index,
                    value: asset.$read_func::<LE>()?,
                })
            }
        }

        simple_property_write!($property_type, $write_func, value, $ty);
    };
}

/// Int8 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int8Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Int8 value
    pub value: i8,
}
impl_property_data_trait!(Int8Property);

/// Byte property value
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub enum BytePropertyValue {
    /// Byte variant
    Byte(u8),
    /// FName variant
    FName(FName),
}

/// Byte property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ByteProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Enum type
    pub enum_type: Option<FName>,
    /// Byte value
    pub value: BytePropertyValue,
}
impl_property_data_trait!(ByteProperty);

/// Bool property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct BoolProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Boolean value
    pub value: bool,
}
impl_property_data_trait!(BoolProperty);

/// Int32 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct IntProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Int32 value
    pub value: i32,
}
impl_property_data_trait!(IntProperty);

/// Int16 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int16Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Int16 value
    pub value: i16,
}
impl_property_data_trait!(Int16Property);

/// Int64 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Int64Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Int64 value
    pub value: i64,
}
impl_property_data_trait!(Int64Property);

/// UInt16 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt16Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// UInt16 value
    pub value: u16,
}
impl_property_data_trait!(UInt16Property);

/// UInt32 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt32Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// UInt32 value
    pub value: u32,
}
impl_property_data_trait!(UInt32Property);

/// UInt64 property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct UInt64Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// UInt64 value
    pub value: u64,
}
impl_property_data_trait!(UInt64Property);

/// Float property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct FloatProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Float value
    pub value: OrderedFloat<f32>,
}
impl_property_data_trait!(FloatProperty);

/// Double property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct DoubleProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Double value
    pub value: OrderedFloat<f64>,
}
impl_property_data_trait!(DoubleProperty);

impl BoolProperty {
    /// Read a `BoolProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let value = asset.read_bool()?;
        let property_guid = optional_guid!(asset, include_header);

        Ok(BoolProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for BoolProperty {
    fn write<Writer: ArchiveWriter>(
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
    /// Read an `Int8Property` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        Ok(Int8Property {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: asset.read_i8()?,
        })
    }
}

impl PropertyTrait for Int8Property {
    fn write<Writer: ArchiveWriter>(
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
    /// Read byte property value
    fn read_value<Reader: ArchiveReader>(
        asset: &mut Reader,
        length: i64,
    ) -> Result<BytePropertyValue, Error> {
        let value = match length {
            1 => Some(BytePropertyValue::Byte(asset.read_u8()?)),
            8 => Some(BytePropertyValue::FName(asset.read_fname()?)),
            0 => {
                let name_map_pointer = asset.read_i32::<LE>()?;
                let name_map_index = asset.read_i32::<LE>()?;

                asset.seek(SeekFrom::Current(-(size_of::<i32>() as i64 * 2)))?;

                let byte_value = if name_map_pointer >= 0
                    && name_map_pointer
                        < asset
                            .get_name_map()
                            .get_ref()
                            .get_name_map_index_list()
                            .len() as i32
                    && name_map_index == 0
                    && !asset.get_name_reference(name_map_index, |name| name.contains('/'))
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
            Error::invalid_file(format!("Invalid length of {length} for ByteProperty"))
        })
    }

    /// Read a `ByteProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
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
            ancestry,
            property_guid,
            duplication_index,
            enum_type,
            value,
        })
    }
}

impl PropertyTrait for ByteProperty {
    fn write<Writer: ArchiveWriter>(
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
            asset.write_property_guid(self.property_guid.as_ref())?;
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
    /// Read a `FloatProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(FloatProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: OrderedFloat(asset.read_f32::<LE>()?),
        })
    }
}

impl PropertyTrait for FloatProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LE>(self.value.0)?;
        Ok(size_of::<f32>())
    }
}

impl DoubleProperty {
    /// Read a `DoubleProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(DoubleProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: OrderedFloat(asset.read_f64::<LE>()?),
        })
    }
}

impl PropertyTrait for DoubleProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f64::<LE>(self.value.0)?;
        Ok(size_of::<f64>())
    }
}

impl_int_property!(IntProperty, read_i32, write_i32, i32);
impl_int_property!(Int16Property, read_i16, write_i16, i16);
impl_int_property!(Int64Property, read_i64, write_i64, i64);
impl_int_property!(UInt16Property, read_u16, write_u16, u16);
impl_int_property!(UInt32Property, read_u32, write_u32, u32);
impl_int_property!(UInt64Property, read_u64, write_u64, u64);
