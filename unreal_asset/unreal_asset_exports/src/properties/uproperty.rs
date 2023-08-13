//! All of Unreal Engine UProperties

use std::fmt::Debug;
use std::hash::Hash;

use byteorder::LE;
use enum_dispatch::enum_dispatch;

use unreal_asset_base::{
    custom_version::{FFrameworkObjectVersion, FReleaseObjectVersion},
    enums::{EArrayDim, ELifetimeCondition},
    flags::EPropertyFlags,
    reader::{ArchiveReader, ArchiveWriter},
    types::{FName, PackageIndex},
    Error, FNameContainer,
};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        /// $prop_name
        #[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $prop_name {
            /// Generic property
            pub generic_property: UGenericProperty
        }

        impl $prop_name {
            /// Read a `$prop_name` from an asset
            pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: UGenericProperty::new(asset)?
                })
            }
        }

        impl UPropertyTrait for $prop_name {
            fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                Ok(())
            }
        }
    };

    (
        $prop_name:ident,
        $(

            $(#[$inner:ident $($args:tt)*])*
            $field_name:ident
        ),*
    ) => {
        /// $prop_name
        #[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $prop_name {
            /// Generic property
            pub generic_property: UGenericProperty,
            $(
                $(#[$inner $($args)*])*
                #[container_ignore]
                pub $field_name: PackageIndex,
            )*
        }

        impl $prop_name {
            /// Read a `$prop_name` from an asset
            pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: UGenericProperty::new(asset)?,
                    $(
                        $field_name: PackageIndex::new(asset.read_i32::<LE>()?),
                    )*
                })
            }
        }

        impl UPropertyTrait for $prop_name {
            fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                $(
                    asset.write_i32::<LE>(self.$field_name.index)?;
                )*
                Ok(())
            }
        }
    }
}

/// This must be implemented for all UProperties
#[enum_dispatch]
pub trait UPropertyTrait: Debug + Clone + PartialEq + Eq + Hash {
    /// Write `UProperty` to an asset
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error>;
}

/// UProperty
#[enum_dispatch(UPropertyTrait)]
#[derive(FNameContainer, Debug, Clone, PartialEq, Hash)]
#[container_nobounds]
pub enum UProperty {
    /// Generic UProperty
    UGenericProperty,
    /// Enum
    UEnumProperty,
    /// Array
    UArrayProperty,
    /// Set
    USetProperty,
    /// Object
    UObjectProperty,
    /// SoftObject
    USoftObjectProperty,
    /// LazyObject
    ULazyObjectProperty,
    /// Class
    UClassProperty,
    /// SoftClass
    USoftClassProperty,
    /// Delegate
    UDelegateProperty,
    /// MulticastDelegate
    UMulticastDelegateProperty,
    /// MulticastInlineDelegate
    UMulticastInlineDelegateProperty,
    /// Interface
    UInterfaceProperty,
    /// Map
    UMapProperty,
    /// Bool
    UBoolProperty,
    /// Byte
    UByteProperty,
    /// Struct
    UStructProperty,
    /// Double
    UDoubleProperty,
    /// Float
    UFloatProperty,
    /// Int
    UIntProperty,
    /// Int8
    UInt8Property,
    /// Int16
    UInt16Property,
    /// Int64
    UInt64Property,
    /// UInt8
    UUInt8Property,
    /// UInt16
    UUInt16Property,
    /// UInt64
    UUInt64Property,
    /// Name
    UNameProperty,
    /// String
    UStrProperty,
}

impl Eq for UProperty {}

impl UProperty {
    /// Read a `UProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        serialized_type: FName,
    ) -> Result<Self, Error> {
        let prop: UProperty = serialized_type.get_content(|ty| {
            Ok::<UProperty, Error>(match ty {
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
                "MulticastInlineDelegateProperty" => {
                    UMulticastInlineDelegateProperty::new(asset)?.into()
                }
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
                _ => UGenericProperty::new(asset)?.into(),
            })
        })?;

        Ok(prop)
    }
}

/// UField
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UField {
    /// Next field package index
    pub next: Option<PackageIndex>,
}

/// Generic UProperty
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UGenericProperty {
    /// UField
    #[container_ignore]
    pub u_field: UField,
    /// Array dimension
    #[container_ignore]
    pub array_dim: EArrayDim,
    /// Property flags
    #[container_ignore]
    pub property_flags: EPropertyFlags,
    /// Replication notify function
    pub rep_notify_func: FName,
    /// Replication condition
    #[container_ignore]
    pub blueprint_replication_condition: Option<ELifetimeCondition>,
}

/// Boolean UProperty
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UBoolProperty {
    /// Generic property
    pub generic_property: UGenericProperty,
    /// Element size
    pub element_size: u8,
    /// Is native boolean
    pub native_bool: bool,
}

impl UField {
    /// Read a `UField` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let next = match asset
            .get_custom_version::<FFrameworkObjectVersion>()
            .version
            < FFrameworkObjectVersion::RemoveUfieldNext as i32
        {
            true => Some(PackageIndex::new(asset.read_i32::<LE>()?)),
            false => None,
        };
        Ok(UField { next })
    }

    /// Write a `UField` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        if asset
            .get_custom_version::<FFrameworkObjectVersion>()
            .version
            < FFrameworkObjectVersion::RemoveUfieldNext as i32
        {
            asset.write_i32::<LE>(
                self.next
                    .ok_or_else(|| {
                        Error::no_data(
                            "FFrameworkObjectVersion < RemoveUfieldNext but no next index present"
                                .to_string(),
                        )
                    })?
                    .index,
            )?;
        }
        Ok(())
    }
}

impl UGenericProperty {
    /// Read a `UGenericProperty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let u_field = UField::new(asset)?;

        let array_dim: EArrayDim = asset.read_i32::<LE>()?.try_into()?;
        let property_flags: EPropertyFlags = EPropertyFlags::from_bits(asset.read_u64::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid property flags".to_string()))?;
        let rep_notify_func = asset.read_fname()?;

        let blueprint_replication_condition: Option<ELifetimeCondition> =
            match asset.get_custom_version::<FReleaseObjectVersion>().version
                >= FReleaseObjectVersion::PropertiesSerializeRepCondition as i32
            {
                true => asset.read_u8()?.try_into().ok(),
                false => None,
            };

        Ok(UGenericProperty {
            u_field,
            array_dim,
            property_flags,
            rep_notify_func,
            blueprint_replication_condition,
        })
    }
}

impl UPropertyTrait for UGenericProperty {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.u_field.write(asset)?;
        asset.write_i32::<LE>(self.array_dim.into())?;
        asset.write_u64::<LE>(self.property_flags.bits())?;
        asset.write_fname(&self.rep_notify_func)?;

        if asset.get_custom_version::<FReleaseObjectVersion>().version
            >= FReleaseObjectVersion::PropertiesSerializeRepCondition as i32
        {
            asset.write_u8(
                self.blueprint_replication_condition.ok_or_else(
                    || Error::no_data("FReleaseObjectVersion >= PropertiesSerializeRepCondition but no blueprint_replication_condition found".to_string())
                )?.into()
            )?;
        }
        Ok(())
    }
}

impl UBoolProperty {
    /// Read a `UBoolProeprty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let generic_property = UGenericProperty::new(asset)?;

        let element_size = asset.read_u8()?;
        let native_bool = asset.read_bool()?;

        Ok(UBoolProperty {
            generic_property,
            element_size,
            native_bool,
        })
    }
}

impl UPropertyTrait for UBoolProperty {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.generic_property.write(asset)?;
        asset.write_u8(self.element_size)?;
        asset.write_bool(self.native_bool)?;
        Ok(())
    }
}

parse_simple_property!(
    UEnumProperty,
    /// Value
    value,
    /// Underlying property
    underlying_prop
);
parse_simple_property!(
    UArrayProperty,
    /// Inner property
    inner
);
parse_simple_property!(
    USetProperty,
    /// Set element
    element_prop
);
parse_simple_property!(
    UObjectProperty,
    /// Class index
    property_class
);
parse_simple_property!(
    USoftObjectProperty,
    /// Class index
    property_class
);
parse_simple_property!(
    ULazyObjectProperty,
    /// Class index
    property_class
);
parse_simple_property!(
    UClassProperty,
    /// Class index
    property_class,
    /// Meta-class index
    meta_class
);
parse_simple_property!(
    USoftClassProperty,
    /// Class index
    property_class,
    /// Meta-class index
    meta_class
);
parse_simple_property!(
    UDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property!(
    UMulticastDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property!(
    UMulticastInlineDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property!(
    UInterfaceProperty,
    /// Interface class index
    interface_class
);
parse_simple_property!(
    UMapProperty,
    /// Key
    key_prop,
    /// Value
    value_prop
);
parse_simple_property!(
    UByteProperty,
    /// Enum value index
    enum_value
);
parse_simple_property!(
    UStructProperty,
    /// Struct value index
    struct_value
);

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
