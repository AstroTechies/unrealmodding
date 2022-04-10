use std::io::{Cursor,};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::Error;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::custom_version::{FFrameworkObjectVersion, FReleaseObjectVersion};
use crate::uasset::enums::{EArrayDim, ELifetimeCondition};
use crate::uasset::flags::{EPropertyFlags};
use crate::uasset::unreal_types::{FName, PackageIndex};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        pub struct $prop_name {
            pub generic_property: UGenericProperty
        }

        impl $prop_name {
            pub fn new(asset: &mut Asset) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: UGenericProperty::new(asset)?
                })
            }
        }

        impl UPropertyTrait for $prop_name {
            fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
                self.generic_property.write(asset, cursor)?;
                Ok(())
            }
        }
    };

    ($prop_name:ident, $($field_name:ident),*) => {
        pub struct $prop_name {
            pub generic_property: UGenericProperty,
            $(
                pub $field_name: PackageIndex,
            )*
        }

        impl $prop_name {
            pub fn new(asset: &mut Asset) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: UGenericProperty::new(asset)?,
                    $(
                        $field_name: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
                    )*
                })
            }
        }

        impl UPropertyTrait for $prop_name {
            fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
                self.generic_property.write(asset, cursor)?;
                $(
                    cursor.write_i32::<LittleEndian>(self.$field_name.index)?;
                )*
                Ok(())
            }
        }
    }
}

#[enum_dispatch]
pub trait UPropertyTrait {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error>;
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
    pub next: Option<PackageIndex>
}

pub struct UGenericProperty {
    pub u_field: UField,
    pub array_dim: EArrayDim,
    pub property_flags: EPropertyFlags,
    pub rep_notify_func: FName,
    pub blueprint_replication_condition: Option<ELifetimeCondition>
}

pub struct UBoolProperty {
    pub generic_property: UGenericProperty,
    pub element_size: u8,
    pub native_bool: bool
}

impl UField {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let next = match asset.get_custom_version::<FFrameworkObjectVersion>().version < FFrameworkObjectVersion::RemoveUfieldNext as i32 {
            true => Some(PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?)),
            false => None
        };
        Ok(UField {
            next
        })
    }

    pub fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        if asset.get_custom_version::<FFrameworkObjectVersion>().version < FFrameworkObjectVersion::RemoveUfieldNext as i32 {
            cursor.write_i32::<LittleEndian>(self.next.ok_or(Error::no_data("FFrameworkObjectVersion < RemoveUfieldNext but no next index present".to_string()))?.index)?;
        }
        Ok(())
    }
}

impl UGenericProperty {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let u_field = UField::new(asset)?;

        let array_dim: EArrayDim = asset.cursor.read_i32::<LittleEndian>()?.try_into()?;
        let property_flags: EPropertyFlags = EPropertyFlags::from_bits(asset.cursor.read_u64::<LittleEndian>()?).ok_or(Error::invalid_file("Invalid property flags".to_string()))?;
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

impl UPropertyTrait for UGenericProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.u_field.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(self.array_dim.into())?;
        cursor.write_u64::<LittleEndian>(self.property_flags.bits())?;
        asset.write_fname(cursor, &self.rep_notify_func)?;

        if asset.get_custom_version::<FReleaseObjectVersion>().version >= FReleaseObjectVersion::PropertiesSerializeRepCondition as i32 {
            cursor.write_u8(self.blueprint_replication_condition.ok_or(Error::no_data("FReleaseObjectVersion >= PropertiesSerializeRepCondition but no blueprint_replication_condition found".to_string()))?.into())?;
        }
        Ok(())
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

impl UPropertyTrait for UBoolProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.generic_property.write(asset, cursor)?;
        cursor.write_u8(self.element_size)?;
        cursor.write_bool(self.native_bool)?;
        Ok(())
    }
}

parse_simple_property!(UEnumProperty, value, underlying_prop);
parse_simple_property!(UArrayProperty, inner);
parse_simple_property!(USetProperty, element_prop);
parse_simple_property!(UObjectProperty, property_class);
parse_simple_property!(USoftObjectProperty, property_class);
parse_simple_property!(ULazyObjectProperty, property_class);
parse_simple_property!(UClassProperty, property_class, meta_class);
parse_simple_property!(USoftClassProperty, property_class, meta_class);
parse_simple_property!(UDelegateProperty, signature_function);
parse_simple_property!(UMulticastDelegateProperty, signature_function);
parse_simple_property!(UMulticastInlineDelegateProperty, signature_function);
parse_simple_property!(UInterfaceProperty, interface_class);
parse_simple_property!(UMapProperty, key_prop, value_prop);
parse_simple_property!(UByteProperty, enum_value);
parse_simple_property!(UStructProperty, struct_value);

parse_simple_property!(UDoubleProperty);
parse_simple_property!(UFloatProperty);
parse_simple_property!(UIntProperty);
parse_simple_property!(UInt8Property);
parse_simple_property!(UInt16Property);
parse_simple_property!(UInt64Property);
parse_simple_property!(UUInt8Property);
parse_simple_property!(UUInt16Property);
parse_simple_property!(UUInt64Property);
parse_simple_property!(UNameProperty);
parse_simple_property!(UStrProperty);