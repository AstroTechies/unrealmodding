use std::io::{Cursor, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::flags::{EArrayDim, ELifetimeCondition, EObjectFlags, EPropertyFlags};
use crate::uasset::unreal_types::{FName, PackageIndex};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(cursor, asset)?
            })
        }
    }
}

macro_rules! parse_simple_property_index {
    ($prop_name:ident, $($index_name:ident),*) => {
        pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(cursor, asset)?,
                $(
                    $index_name: PackageIndex::new(cursor.read_i32::<LittleEndian>()?),
                )*
            })
        }
    };
}

macro_rules! parse_simple_property_prop {
    ($prop_name:ident, $($prop:ident),*) => {
        pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: FGenericProperty::new(cursor, asset)?,
                $(
                    $prop: Box::new(FProperty::new(cursor, asset)?),
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
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let serialized_type = asset.read_fname()?;
        let res: FProperty = match serialized_type.content.as_str() {
            "EnumProperty" => FEnumProperty::new(cursor, asset)?.into(),
            "ArrayProperty" => FArrayProperty::new(cursor, asset)?.into(),
            "SetProperty" => FSetProperty::new(cursor, asset)?.into(),
            "ObjectProperty" => FObjectProperty::new(cursor, asset)?.into(),
            "SoftObjectProperty" => FSoftObjectProperty::new(cursor, asset)?.into(),
            "ClassProperty" => FClassProperty::new(cursor, asset)?.into(),
            "SoftClassProperty" => FSoftClassProperty::new(cursor, asset)?.into(),
            "DelegateProperty" => FDelegateProperty::new(cursor, asset)?.into(),
            "MulticastDelegateProperty" => FMulticastDelegateProperty::new(cursor, asset)?.into(),
            "MulticastInlineDelegateProperty" => FMulticastInlineDelegateProperty::new(cursor, asset)?.into(),
            "InterfaceProperty" => FInterfaceProperty::new(cursor, asset)?.into(),
            "MapProperty" => FMapProperty::new(cursor, asset)?.into(),
            "BoolProperty" => FBoolProperty::new(cursor, asset)?.into(),
            "ByteProperty" => FByteProperty::new(cursor, asset)?.into(),
            "StructProperty" => FStructProperty::new(cursor, asset)?.into(),
            "NumericProperty" => FNumericProperty::new(cursor, asset)?.into(),
            _ => FGenericProperty::new(cursor, asset)?.into()
        };

        Ok(res)
    }
}

pub struct FGenericProperty {
    name: FName,
    flags: EObjectFlags,
    array_dim: EArrayDim,
    element_size: i32,
    property_flags: EPropertyFlags,
    rep_index: u16,
    rep_notify_func: FName,
    blueprint_replication_condition: ELifetimeCondition
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
    generic_property: FGenericProperty,
    key_prop: Box<FProperty>,
    value_prop: Box<FProperty>
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
    generic_property: FGenericProperty,
    struct_value: PackageIndex
}

pub struct FNumericProperty { generic_property: FGenericProperty }

impl FGenericProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let name = asset.read_fname()?;
        let flags: EObjectFlags = cursor.read_u32::<LittleEndian>()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid object flags"))?;
        let array_dim : EArrayDim = cursor.read_i32::<LittleEndian>()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid array dim"))?;
        let element_size = cursor.read_i32::<LittleEndian>()?;
        let property_flags: EPropertyFlags = cursor.read_u64::<LittleEndian>()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid property flags"))?;
        let rep_index = cursor.read_u16::<LittleEndian>()?;
        let rep_notify_func = asset.read_fname()?;
        let blueprint_replication_condition: ELifetimeCondition = cursor.read_u8()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid blueprint replication condition"))?;

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
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(cursor, asset)?;
        let enum_value = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);
        let underlying_prop = FProperty::new(cursor, asset)?;

        Ok(FEnumProperty {
            generic_property,
            enum_value,
            underlying_prop: Box::new(underlying_prop)
        })
    }
}

impl FBoolProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(cursor, asset)?;
        let field_size = cursor.read_u8()?;
        let byte_offset = cursor.read_u8()?;
        let byte_mask = cursor.read_u8()?;
        let field_mask = cursor.read_u8()?;
        let native_bool = cursor.read_bool()?;
        let value = cursor.read_bool()?;

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
