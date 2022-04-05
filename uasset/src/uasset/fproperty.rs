use std::io::{Cursor, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::enums::{ELifetimeCondition, EArrayDim};
use crate::uasset::flags::{EObjectFlags, EPropertyFlags};
use crate::uasset::unreal_types::{FName, PackageIndex};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        pub fn new(asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(asset)?
            })
        }
    }
}

macro_rules! parse_simple_property_index {
    ($prop_name:ident, $($index_name:ident),*) => {
        pub fn new(asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(asset)?,
                $(
                    $index_name: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
                )*
            })
        }
    };
}

macro_rules! parse_simple_property_prop {
    ($prop_name:ident, $($prop:ident),*) => {
        pub fn new(asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(asset)?,
                $(
                    $prop: Box::new(FProperty::new(asset)?),
                )*
            })
        }
    };
}

#[enum_dispatch]
pub trait FPropertyTrait {
}

#[enum_dispatch(FPropertyTrait)]
pub enum FProperty {
    FGenericProperty,
    FEnumProperty,
    FArrayProperty,
    FSetProperty,
    FObjectProperty,
    FSoftObjectProperty,
    FClassProperty,
    FSoftClassProperty,
    FDelegateProperty,
    FMulticastDelegateProperty,
    FMulticastInlineDelegateProperty,
    FInterfaceProperty,
    FMapProperty,
    FBoolProperty,
    FByteProperty,
    FStructProperty,
    FNumericProperty
}

impl FProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let serialized_type = asset.read_fname()?;
        let res: FProperty = match serialized_type.content.as_str() {
            "EnumProperty" => FEnumProperty::new(asset)?.into(),
            "ArrayProperty" => FArrayProperty::new(asset)?.into(),
            "SetProperty" => FSetProperty::new(asset)?.into(),
            "ObjectProperty" => FObjectProperty::new(asset)?.into(),
            "SoftObjectProperty" => FSoftObjectProperty::new(asset)?.into(),
            "ClassProperty" => FClassProperty::new(asset)?.into(),
            "SoftClassProperty" => FSoftClassProperty::new(asset)?.into(),
            "DelegateProperty" => FDelegateProperty::new(asset)?.into(),
            "MulticastDelegateProperty" => FMulticastDelegateProperty::new(asset)?.into(),
            "MulticastInlineDelegateProperty" => FMulticastInlineDelegateProperty::new(asset)?.into(),
            "InterfaceProperty" => FInterfaceProperty::new(asset)?.into(),
            "MapProperty" => FMapProperty::new(asset)?.into(),
            "BoolProperty" => FBoolProperty::new(asset)?.into(),
            "ByteProperty" => FByteProperty::new(asset)?.into(),
            "StructProperty" => FStructProperty::new(asset)?.into(),
            "NumericProperty" => FNumericProperty::new(asset)?.into(),
            _ => FGenericProperty::new(asset)?.into()
        };

        Ok(res)
    }
}

pub struct FGenericProperty {
    pub name: FName,
    pub flags: EObjectFlags,
    pub array_dim: EArrayDim,
    pub element_size: i32,
    pub property_flags: EPropertyFlags,
    pub rep_index: u16,
    pub rep_notify_func: FName,
    pub blueprint_replication_condition: ELifetimeCondition
}

pub struct FEnumProperty {
    generic_property: FGenericProperty,
    enum_value: PackageIndex,
    underlying_prop: Box<FProperty>
}

pub struct FArrayProperty {
    generic_property: FGenericProperty,
    inner: Box<FProperty>
}

pub struct FSetProperty {
    generic_property: FGenericProperty,
    element_prop: Box<FProperty>
}

pub struct FObjectProperty {
    generic_property: FGenericProperty,
    property_class: PackageIndex
}

pub struct FSoftObjectProperty {
    generic_property: FGenericProperty,
    property_class: PackageIndex
}

pub struct FClassProperty {
    generic_property: FGenericProperty,
    property_class: PackageIndex,
    meta_class: PackageIndex
}

pub struct FSoftClassProperty {
    generic_property: FGenericProperty,
    property_class: PackageIndex,
    meta_class: PackageIndex
}

pub struct FDelegateProperty {
    generic_property: FGenericProperty,
    signature_function: PackageIndex
}

pub struct FMulticastDelegateProperty {
    generic_property: FGenericProperty,
    signature_function: PackageIndex
}

pub struct FMulticastInlineDelegateProperty {
    generic_property: FGenericProperty,
    signature_function: PackageIndex
}

pub struct FInterfaceProperty {
    generic_property: FGenericProperty,
    interface_class: PackageIndex
}

pub struct FMapProperty {
    pub generic_property: FGenericProperty,
    pub key_prop: Box<FProperty>,
    pub value_prop: Box<FProperty>
}

pub struct FBoolProperty {
    generic_property: FGenericProperty,

    field_size: u8,
    byte_offset: u8,
    byte_mask: u8,
    field_mask: u8,
    native_bool: bool,
    value: bool
}

pub struct FByteProperty {
    generic_property: FGenericProperty,
    enum_value: PackageIndex
}

pub struct FStructProperty {
    pub generic_property: FGenericProperty,
    pub struct_value: PackageIndex
}

pub struct FNumericProperty { generic_property: FGenericProperty }

impl FGenericProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        
        let name = asset.read_fname()?;
        let flags: EObjectFlags = EObjectFlags::from_bits(asset.cursor.read_u32::<LittleEndian>()?).ok_or(Error::new(ErrorKind::Other, "Invalid object flags"))?;
        let array_dim : EArrayDim = asset.cursor.read_i32::<LittleEndian>()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid array dim"))?;
        let element_size = asset.cursor.read_i32::<LittleEndian>()?;
        let property_flags: EPropertyFlags = EPropertyFlags::from_bits(asset.cursor.read_u64::<LittleEndian>()?).ok_or(Error::new(ErrorKind::Other, "Invalid property flags"))?;
        let rep_index = asset.cursor.read_u16::<LittleEndian>()?;
        let rep_notify_func = asset.read_fname()?;
        let blueprint_replication_condition: ELifetimeCondition = asset.cursor.read_u8()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid blueprint replication condition"))?;

        Ok(FGenericProperty {
            name,
            flags,
            array_dim,
            element_size,
            property_flags,
            rep_index,
            rep_notify_func,
            blueprint_replication_condition
        })
    }
}

impl FEnumProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(asset)?;
        let enum_value = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let underlying_prop = FProperty::new(asset)?;

        Ok(FEnumProperty {
            generic_property,
            enum_value,
            underlying_prop: Box::new(underlying_prop)
        })
    }
}

impl FBoolProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(asset)?;
        let field_size = asset.cursor.read_u8()?;
        let byte_offset = asset.cursor.read_u8()?;
        let byte_mask = asset.cursor.read_u8()?;
        let field_mask = asset.cursor.read_u8()?;
        let native_bool = asset.cursor.read_bool()?;
        let value = asset.cursor.read_bool()?;

        Ok(FBoolProperty {
            generic_property,
            field_size,
            byte_offset,
            byte_mask,
            field_mask,
            native_bool,
            value
        })
    }
}

impl FArrayProperty { parse_simple_property_prop!(FArrayProperty, inner); }
impl FSetProperty { parse_simple_property_prop!(FSetProperty, element_prop); }
impl FObjectProperty { parse_simple_property_index!(FObjectProperty, property_class); }
impl FSoftObjectProperty { parse_simple_property_index!(FSoftObjectProperty, property_class); }
impl FClassProperty { parse_simple_property_index!(FClassProperty, property_class, meta_class); }
impl FSoftClassProperty { parse_simple_property_index!(FSoftClassProperty, property_class, meta_class); }
impl FDelegateProperty { parse_simple_property_index!(FDelegateProperty, signature_function); }
impl FMulticastDelegateProperty { parse_simple_property_index!(FMulticastDelegateProperty, signature_function); }
impl FMulticastInlineDelegateProperty { parse_simple_property_index!(FMulticastInlineDelegateProperty, signature_function); }
impl FInterfaceProperty { parse_simple_property_index!(FInterfaceProperty, interface_class); }
impl FMapProperty { parse_simple_property_prop!(FMapProperty, key_prop, value_prop); }
impl FByteProperty { parse_simple_property_index!(FByteProperty, enum_value); }
impl FStructProperty { parse_simple_property_index!(FStructProperty, struct_value); }
impl FNumericProperty { parse_simple_property!(FNumericProperty); }
