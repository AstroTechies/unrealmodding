use std::io::{Cursor, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::custom_version::{FFrameworkObjectVersion, FReleaseObjectVersion};
use crate::uasset::enums::{EArrayDim, ELifetimeCondition};
use crate::uasset::flags::{EPropertyFlags};
use crate::uasset::unreal_types::{FName, PackageIndex};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        pub fn new(asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: UGenericProperty::new(asset)?
            })
        }
    };

    ($prop_name:ident, $($field_name:ident),*) => {
        pub fn new(asset: &mut Asset) -> Result<Self, Error> {
            Ok($prop_name {
                generic_property: UGenericProperty::new(asset)?,
                $(
                    $field_name: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
                )*
            })
        }
    }
}

#[enum_dispatch]
pub trait UPropertyTrait {
}

#[enum_dispatch(UPropertyTrait)]
pub enum UProperty {
    UGenericProperty,
    UEnumProperty,
    UArrayProperty,
    USetProperty,
    UObjectProperty,
    USoftObjectProperty,
    ULazyObjectProperty,
    UClassProperty,
    USoftClassProperty,
    UDelegateProperty,
    UMulticastDelegateProperty,
    UMulticastInlineDelegateProperty,
    UInterfaceProperty,
    UMapProperty,
    UBoolProperty,
    UByteProperty,
    UStructProperty,
    UDoubleProperty,
    UFloatProperty,
    UIntProperty,
    UInt8Property,
    UInt16Property,
    UInt64Property,
    UUInt8Property,
    UUInt16Property,
    UUInt64Property,
    UNameProperty,
    UStrProperty,
}

impl UProperty {
    pub fn new(asset: &mut Asset, serialized_type: FName) -> Result<Self, Error> {
        let prop: UProperty = match serialized_type.content.as_str() {
            "EnumProperty" => UEnumProperty::new(asset)?.into(),
            "ArrayProperty" => UArrayProperty::new(asset)?.into(),
            "SetProperty" => USetProperty::new(asset)?.into(),
            "ObjectProperty" => UObjectProperty::new(asset)?.into(),
            "SoftObjectProperty" => USoftObjectProperty::new(asset)?.into(),
            "LazyObjectProperty" => ULazyObjectProperty::new(asset)?.into(),
            "ClassProperty" => UClassProperty::new(asset)?.into(),
            "SoftClassProperty" => USoftClassProperty::new(asset)?.into(),
            "DelegateProperty" => UDelegateProperty::new(asset)?.into(),
            "MulticastDelegateProperty" => UMulticastDelegateProperty::new(asset)?.into(),
            "MulticastInlineDelegateProperty" => UMulticastInlineDelegateProperty::new(asset)?.into(),
            "InterfaceProperty" => UInterfaceProperty::new(asset)?.into(),
            "MapProperty" => UMapProperty::new(asset)?.into(),
            "ByteProperty" => UByteProperty::new(asset)?.into(),
            "StructProperty" => UStructProperty::new(asset)?.into(),
            "DoubleProperty" => UDoubleProperty::new(asset)?.into(),
            "FloatProperty" => UFloatProperty::new(asset)?.into(),
            "IntProperty" => UIntProperty::new(asset)?.into(),
            "Int8Property" => UInt8Property::new(asset)?.into(),
            "Int16Property" => UInt16Property::new(asset)?.into(),
            "Int64Property" => UInt64Property::new(asset)?.into(),
            "UInt8Property" => UUInt8Property::new(asset)?.into(),
            "UInt16Property" => UUInt16Property::new(asset)?.into(),
            "UInt64Property" => UUInt64Property::new(asset)?.into(),
            "NameProperty" => UNameProperty::new(asset)?.into(),
            "StrProperty" => UStrProperty::new(asset)?.into(),
            _ => UGenericProperty::new(asset)?.into()
        };

        Ok(prop)
    }
}

pub struct UField {
    next: Option<PackageIndex>
}

pub struct UGenericProperty {
    u_field: UField,
    array_dim: EArrayDim,
    property_flags: EPropertyFlags,
    rep_notify_func: FName,
    blueprint_replication_condition: Option<ELifetimeCondition>
}

pub struct UEnumProperty {
    generic_property: UGenericProperty,
    value: PackageIndex,
    underlying_prop: PackageIndex
}

pub struct UArrayProperty {
    generic_property: UGenericProperty,
    inner: PackageIndex
}

pub struct USetProperty {
    generic_property: UGenericProperty,
    element_prop: PackageIndex
}

pub struct UObjectProperty {
    generic_property: UGenericProperty,
    property_class: PackageIndex
}

pub struct USoftObjectProperty {
    generic_property: UGenericProperty,
    property_class: PackageIndex
}

pub struct ULazyObjectProperty {
    generic_property: UGenericProperty,
    property_class: PackageIndex
}

pub struct UClassProperty {
    generic_property: UGenericProperty,
    property_class: PackageIndex,
    meta_class: PackageIndex
}

pub struct USoftClassProperty {
    generic_property: UGenericProperty,
    property_class: PackageIndex,
    meta_class: PackageIndex
}

pub struct UDelegateProperty {
    generic_property: UGenericProperty,
    signature_function: PackageIndex
}

pub struct UMulticastDelegateProperty {
    generic_property: UGenericProperty,
    signature_function: PackageIndex
}

pub struct UMulticastInlineDelegateProperty {
    generic_property: UGenericProperty,
    signature_function: PackageIndex
}

pub struct UInterfaceProperty {
    generic_property: UGenericProperty,
    interface_class: PackageIndex
}

pub struct UMapProperty {
    generic_property: UGenericProperty,
    key_prop: PackageIndex,
    value_prop: PackageIndex
}

pub struct UBoolProperty {
    generic_property: UGenericProperty,
    element_size: u8,
    native_bool: bool
}

pub struct UByteProperty {
    generic_property: UGenericProperty,
    enum_value: PackageIndex
}

pub struct UStructProperty {
    generic_property: UGenericProperty,
    struct_value: PackageIndex
}

pub struct UDoubleProperty { generic_property: UGenericProperty }
pub struct UFloatProperty { generic_property: UGenericProperty }
pub struct UIntProperty { generic_property: UGenericProperty }
pub struct UInt8Property { generic_property: UGenericProperty }
pub struct UInt16Property { generic_property: UGenericProperty }
pub struct UInt64Property { generic_property: UGenericProperty }
pub struct UUInt8Property { generic_property: UGenericProperty }
pub struct UUInt16Property { generic_property: UGenericProperty }
pub struct UUInt64Property { generic_property: UGenericProperty }
pub struct UNameProperty { generic_property: UGenericProperty }
pub struct UStrProperty { generic_property: UGenericProperty }

impl UField {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        
        let next = match asset.get_custom_version::<FFrameworkObjectVersion>().version < FFrameworkObjectVersion::RemoveUField_Next as i32 {
            true => Some(PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?)),
            false => None
        };
        Ok(UField {
            next
        })
    }
}

impl UGenericProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let u_field = UField::new(asset)?;

        let array_dim: EArrayDim = asset.cursor.read_i32::<LittleEndian>()?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid array dim"))?;
        let property_flags: EPropertyFlags = EPropertyFlags::from_bits(asset.cursor.read_u64::<LittleEndian>()?).ok_or(Error::new(ErrorKind::Other, "Invalid property flags"))?;
        let rep_notify_func = asset.read_fname()?;

        let blueprint_replication_condition: Option<ELifetimeCondition> = match asset.get_custom_version::<FReleaseObjectVersion>().version >= FReleaseObjectVersion::PropertiesSerializeRepCondition as i32 {
            true => asset.cursor.read_u8()?.try_into().ok(),
            false => None
        };

        Ok(UGenericProperty {
            u_field,
            array_dim,
            property_flags,
            rep_notify_func,
            blueprint_replication_condition
        })
    }
}

impl UBoolProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        
        let generic_property = UGenericProperty::new(asset)?;

        let element_size = asset.cursor.read_u8()?;
        let native_bool = asset.cursor.read_bool()?;

        Ok(UBoolProperty {
            generic_property,
            element_size,
            native_bool
        })
    }
}

impl UEnumProperty { parse_simple_property!(UEnumProperty, value, underlying_prop); }
impl UArrayProperty { parse_simple_property!(UArrayProperty, inner); }
impl USetProperty { parse_simple_property!(USetProperty, element_prop); }
impl UObjectProperty { parse_simple_property!(UObjectProperty, property_class); }
impl USoftObjectProperty { parse_simple_property!(USoftObjectProperty, property_class); }
impl ULazyObjectProperty { parse_simple_property!(ULazyObjectProperty, property_class); }
impl UClassProperty { parse_simple_property!(UClassProperty, property_class, meta_class); }
impl USoftClassProperty { parse_simple_property!(USoftClassProperty, property_class, meta_class); }
impl UDelegateProperty { parse_simple_property!(UDelegateProperty, signature_function); }
impl UMulticastDelegateProperty { parse_simple_property!(UMulticastDelegateProperty, signature_function); }
impl UMulticastInlineDelegateProperty { parse_simple_property!(UMulticastInlineDelegateProperty, signature_function); }
impl UInterfaceProperty { parse_simple_property!(UInterfaceProperty, interface_class); }
impl UMapProperty { parse_simple_property!(UMapProperty, key_prop, value_prop); }
impl UByteProperty { parse_simple_property!(UByteProperty, enum_value); }
impl UStructProperty { parse_simple_property!(UStructProperty, struct_value); }

impl UDoubleProperty { parse_simple_property!(UDoubleProperty); }
impl UFloatProperty { parse_simple_property!(UFloatProperty); }
impl UIntProperty { parse_simple_property!(UIntProperty); }
impl UInt8Property { parse_simple_property!(UInt8Property); }
impl UInt16Property { parse_simple_property!(UInt16Property); }
impl UInt64Property { parse_simple_property!(UInt64Property); }
impl UUInt8Property { parse_simple_property!(UUInt8Property); }
impl UUInt16Property { parse_simple_property!(UUInt16Property); }
impl UUInt64Property { parse_simple_property!(UUInt64Property); }
impl UNameProperty { parse_simple_property!(UNameProperty); }
impl UStrProperty { parse_simple_property!(UStrProperty); }