use enum_dispatch::enum_dispatch;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{fmt::Debug, hash::Hash};

use crate::{error::Error, inner_trait};

use self::{
    array_property::UsmapArrayPropertyData, enum_property::UsmapEnumPropertyData,
    map_property::UsmapMapPropertyData, set_property::UsmapSetPropertyData,
    shallow_property::UsmapShallowPropertyData, struct_property::UsmapStructPropertyData,
};

use super::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter};

pub mod array_property;
pub mod enum_property;
pub mod map_property;
pub mod set_property;
pub mod shallow_property;
pub mod struct_property;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EPropertyType {
    ByteProperty,
    BoolProperty,
    IntProperty,
    FloatProperty,
    ObjectProperty,
    NameProperty,
    DelegateProperty,
    DoubleProperty,
    ArrayProperty,
    StructProperty,
    StrProperty,
    TextProperty,
    InterfaceProperty,
    MulticastDelegateProperty,
    WeakObjectProperty,  //
    LazyObjectProperty,  // When deserialized, these 3 properties will be SoftObjects
    AssetObjectProperty, //
    SoftObjectProperty,
    UInt64Property,
    UInt32Property,
    UInt16Property,
    Int64Property,
    Int16Property,
    Int8Property,
    MapProperty,
    SetProperty,
    EnumProperty,
    FieldPathProperty,

    Unknown = 0xFF,
}

#[enum_dispatch]
pub trait UsmapPropertyDataTrait: Debug + Hash + Clone + PartialEq + Eq {
    fn write<Writer: UsmapWriter>(&self, writer: &mut Writer) -> Result<usize, Error>;
}

#[enum_dispatch(UsmapPropertyDataTrait)]
pub enum UsmapPropertyData {
    UsmapEnumPropertyData,
    UsmapStructPropertyData,
    UsmapSetPropertyData,
    UsmapArrayPropertyData,
    UsmapMapPropertyData,

    UsmapShallowPropertyData,
}

impl UsmapPropertyData {
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let prop_type: EPropertyType = EPropertyType::try_from(asset.read_u8()?)?;

        let res: UsmapPropertyData = match prop_type {
            EPropertyType::ArrayProperty => UsmapArrayPropertyData::new(asset)?.into(),
            EPropertyType::StructProperty => UsmapStructPropertyData::new(asset)?.into(),
            EPropertyType::MapProperty => UsmapMapPropertyData::new(asset)?.into(),
            EPropertyType::SetProperty => UsmapSetPropertyData::new(asset)?.into(),
            EPropertyType::EnumProperty => UsmapEnumPropertyData::new(asset)?.into(),
            _ => UsmapShallowPropertyData {
                property_type: prop_type,
            }
            .into(),
        };

        Ok(res)
    }
}

inner_trait!(
    UsmapPropertyData,
    UsmapEnumPropertyData,
    UsmapStructPropertyData,
    UsmapSetPropertyData,
    UsmapArrayPropertyData,
    UsmapMapPropertyData,
    UsmapShallowPropertyData
);

impl Eq for UsmapPropertyData {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapProperty {
    pub name: String,
    pub schema_index: u16,
    pub array_size: u8,
    pub property_data: UsmapPropertyData,
}

impl UsmapProperty {
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let schema_index = asset.read_u16()?;
        let array_size = asset.read_u8()?;
        let name = asset.read_name()?;

        let property_data = UsmapPropertyData::new(asset)?;
        Ok(UsmapProperty {
            name,
            schema_index,
            array_size,
            property_data,
        })
    }
}
