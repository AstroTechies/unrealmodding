//! All of Unreal Engine FProperties

use std::fmt::Debug;
use std::hash::Hash;

use byteorder::LE;
use enum_dispatch::enum_dispatch;
use unreal_asset_proc_macro::FNameContainer;

use crate::enums::{EArrayDim, ELifetimeCondition};
use crate::error::Error;
use crate::flags::{EObjectFlags, EPropertyFlags};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{
    fname::{FName, ToSerializedName},
    PackageIndex,
};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        /// $prop_name
        #[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $prop_name {
            /// Generic property
            pub generic_property: FGenericProperty,
        }

        impl $prop_name {
            /// Read an `$prop_name` from an asset
            pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                Ok(())
            }
        }
    };
}

macro_rules! parse_simple_property_index {
    (
        $prop_name:ident,
        $(
            $(#[$inner:ident $($args:tt)*])*
            $index_name:ident
        ),*
    ) => {
        /// $prop_name
        #[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $prop_name {
            /// Generic property
            pub generic_property: FGenericProperty,
            $(
                $(#[$inner $($args)*])*
                #[container_ignore]
                pub $index_name: PackageIndex,
            )*
        }

        impl $prop_name {
            /// Read an `$prop_name` from an asset
            pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                    $(
                        $index_name: PackageIndex::new(asset.read_i32::<LE>()?),
                    )*
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                $(
                    asset.write_i32::<LE>(self.$index_name.index)?;
                )*
                Ok(())
            }
        }
    };
}

macro_rules! parse_simple_property_prop {
    (
        $prop_name:ident,
        $(
            $(#[$inner:ident $($args:tt)*])*
            $prop:ident
        ),*
    ) => {
        /// $prop_name
        #[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $prop_name {
            /// Generic property
            pub generic_property: FGenericProperty,
            $(
                $(#[$inner $($args)*])*
                pub $prop: Box<FProperty>,
            )*
        }

        impl $prop_name {
            /// Read an `$prop_name` from an asset
            pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                    $(
                        $prop: Box::new(FProperty::new(asset)?),
                    )*
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                $(
                    FProperty::write(self.$prop.as_ref(), asset)?;
                )*
                Ok(())
            }
        }
    };
}

/// This must be implemented for all FProperties
#[enum_dispatch]
pub trait FPropertyTrait: Debug + Clone + PartialEq + Eq + Hash {
    /// Write `FProperty` to an asset
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error>;
}

/// FProperty
#[enum_dispatch(FPropertyTrait)]
#[derive(FNameContainer, Hash, PartialEq, Clone, Debug)]
#[container_nobounds]
pub enum FProperty {
    /// Generic FProperty
    FGenericProperty,
    /// Enum
    FEnumProperty,
    /// Array
    FArrayProperty,
    /// Set
    FSetProperty,
    /// Object
    FObjectProperty,
    /// SoftObject
    FSoftObjectProperty,
    /// Class
    FClassProperty,
    /// SoftClass
    FSoftClassProperty,
    /// Delegate
    FDelegateProperty,
    /// MulticastDelegate
    FMulticastDelegateProperty,
    /// MulticastInlineDelegate
    FMulticastInlineDelegateProperty,
    /// Interface
    FInterfaceProperty,
    /// Map
    FMapProperty,
    /// Bool
    FBoolProperty,
    /// Byte
    FByteProperty,
    /// Struct
    FStructProperty,
    /// Numeric
    FNumericProperty,
}

impl Eq for FProperty {}

impl FProperty {
    /// Read an `FProperty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let serialized_type = asset.read_fname()?;
        let res: FProperty = match serialized_type.get_content() {
            "EnumProperty" => FEnumProperty::new(asset)?.into(),
            "ArrayProperty" => FArrayProperty::new(asset)?.into(),
            "SetProperty" => FSetProperty::new(asset)?.into(),
            "ObjectProperty" => FObjectProperty::new(asset)?.into(),
            "SoftObjectProperty" => FSoftObjectProperty::new(asset)?.into(),
            "ClassProperty" => FClassProperty::new(asset)?.into(),
            "SoftClassProperty" => FSoftClassProperty::new(asset)?.into(),
            "DelegateProperty" => FDelegateProperty::new(asset)?.into(),
            "MulticastDelegateProperty" => FMulticastDelegateProperty::new(asset)?.into(),
            "MulticastInlineDelegateProperty" => {
                FMulticastInlineDelegateProperty::new(asset)?.into()
            }
            "InterfaceProperty" => FInterfaceProperty::new(asset)?.into(),
            "MapProperty" => FMapProperty::new(asset)?.into(),
            "BoolProperty" => FBoolProperty::new(asset)?.into(),
            "ByteProperty" => FByteProperty::new(asset)?.into(),
            "StructProperty" => FStructProperty::new(asset)?.into(),
            "NumericProperty" => FNumericProperty::new(asset)?.into(),
            _ => FGenericProperty::with_serialized_type(asset, Some(serialized_type))?.into(),
        };

        Ok(res)
    }

    /// Write an `FProperty` to an asset
    pub fn write<Writer: ArchiveWriter>(
        property: &FProperty,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        let property_serialized_name = property.to_serialized_name();
        asset.write_fname(
            &asset
                .get_name_map()
                .get_mut()
                .add_fname(&property_serialized_name),
        )?;
        property.write(asset)
    }
}

impl ToSerializedName for FProperty {
    fn to_serialized_name(&self) -> String {
        match self {
            FProperty::FEnumProperty(_) => String::from("EnumProperty"),
            FProperty::FArrayProperty(_) => String::from("ArrayProperty"),
            FProperty::FSetProperty(_) => String::from("SetProperty"),
            FProperty::FObjectProperty(_) => String::from("ObjectProperty"),
            FProperty::FSoftObjectProperty(_) => String::from("SoftObjectProperty"),
            FProperty::FClassProperty(_) => String::from("ClassProperty"),
            FProperty::FSoftClassProperty(_) => String::from("SoftClassProperty"),
            FProperty::FDelegateProperty(_) => String::from("DelegateProperty"),
            FProperty::FMulticastDelegateProperty(_) => String::from("MulticastDelegateProperty"),
            FProperty::FMulticastInlineDelegateProperty(_) => {
                String::from("MulticastInlineDelegateProperty")
            }
            FProperty::FInterfaceProperty(_) => String::from("InterfaceProperty"),
            FProperty::FMapProperty(_) => String::from("MapProperty"),
            FProperty::FBoolProperty(_) => String::from("BoolProperty"),
            FProperty::FByteProperty(_) => String::from("ByteProperty"),
            FProperty::FStructProperty(_) => String::from("StructProperty"),
            FProperty::FNumericProperty(_) => String::from("NumericProperty"),
            FProperty::FGenericProperty(generic) => generic
                .serialized_type
                .as_ref()
                .map(|e| e.get_content())
                .unwrap_or("Generic")
                .to_string(),
        }
    }
}

/// Generic FProperty
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FGenericProperty {
    /// Property name
    pub name: FName,
    /// Object flags
    #[container_ignore]
    pub flags: EObjectFlags,
    /// Array dimension
    #[container_ignore]
    pub array_dim: EArrayDim,
    /// Array element size
    pub element_size: i32,
    /// Property flags
    #[container_ignore]
    pub property_flags: EPropertyFlags,
    /// Replication index
    pub rep_index: u16,
    /// Replication notify function
    pub rep_notify_func: FName,
    /// Replication condition
    #[container_ignore]
    pub blueprint_replication_condition: ELifetimeCondition,
    /// Serialized type
    pub serialized_type: Option<FName>,
}

/// Enum FProperty
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FEnumProperty {
    /// Generic property
    generic_property: FGenericProperty,
    /// Enum value
    #[container_ignore]
    enum_value: PackageIndex,
    /// Underlying property
    underlying_prop: Box<FProperty>,
}

/// Boolean FProperty
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FBoolProperty {
    /// Generic property
    generic_property: FGenericProperty,

    /// Field size
    field_size: u8,
    /// Byte offset
    byte_offset: u8,
    /// Byte mask
    byte_mask: u8,
    /// Field mask
    field_mask: u8,
    /// Is native boolean
    native_bool: bool,
    /// Value
    value: bool,
}

impl FGenericProperty {
    /// Read an `FGenericProperty` from an asset with a serialized type
    pub fn with_serialized_type<Reader: ArchiveReader>(
        asset: &mut Reader,
        serialized_type: Option<FName>,
    ) -> Result<Self, Error> {
        let name = asset.read_fname()?;
        let flags: EObjectFlags = EObjectFlags::from_bits(asset.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid object flags".to_string()))?; // todo: maybe other error type than invalid_file?
        let array_dim: EArrayDim = asset.read_i32::<LE>()?.try_into()?;
        let element_size = asset.read_i32::<LE>()?;
        let property_flags: EPropertyFlags = EPropertyFlags::from_bits(asset.read_u64::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid property flags".to_string()))?;
        let rep_index = asset.read_u16::<LE>()?;
        let rep_notify_func = asset.read_fname()?;
        let blueprint_replication_condition: ELifetimeCondition = asset.read_u8()?.try_into()?;

        Ok(FGenericProperty {
            name,
            flags,
            array_dim,
            element_size,
            property_flags,
            rep_index,
            rep_notify_func,
            blueprint_replication_condition,
            serialized_type,
        })
    }

    /// Read an `FGenericProperty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        FGenericProperty::with_serialized_type(asset, None)
    }
}

impl FPropertyTrait for FGenericProperty {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_fname(&self.name)?;
        asset.write_u32::<LE>(self.flags.bits())?;
        asset.write_i32::<LE>(self.array_dim.into())?;
        asset.write_i32::<LE>(self.element_size)?;
        asset.write_u64::<LE>(self.property_flags.bits())?;
        asset.write_u16::<LE>(self.rep_index)?;
        asset.write_fname(&self.rep_notify_func)?;
        asset.write_u8(self.blueprint_replication_condition.into())?;
        Ok(())
    }
}

impl FEnumProperty {
    /// Read an `FEnumProperty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(asset)?;
        let enum_value = PackageIndex::new(asset.read_i32::<LE>()?);
        let underlying_prop = FProperty::new(asset)?;

        Ok(FEnumProperty {
            generic_property,
            enum_value,
            underlying_prop: Box::new(underlying_prop),
        })
    }
}

impl FPropertyTrait for FEnumProperty {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.generic_property.write(asset)?;
        asset.write_i32::<LE>(self.enum_value.index)?;
        FProperty::write(self.underlying_prop.as_ref(), asset)?;
        Ok(())
    }
}

impl FBoolProperty {
    /// Read an `FBoolProperty` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(asset)?;
        let field_size = asset.read_u8()?;
        let byte_offset = asset.read_u8()?;
        let byte_mask = asset.read_u8()?;
        let field_mask = asset.read_u8()?;
        let native_bool = asset.read_bool()?;
        let value = asset.read_bool()?;

        Ok(FBoolProperty {
            generic_property,
            field_size,
            byte_offset,
            byte_mask,
            field_mask,
            native_bool,
            value,
        })
    }
}

impl FPropertyTrait for FBoolProperty {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.generic_property.write(asset)?;
        asset.write_u8(self.field_size)?;
        asset.write_u8(self.byte_offset)?;
        asset.write_u8(self.byte_mask)?;
        asset.write_u8(self.field_mask)?;
        asset.write_bool(self.native_bool)?;
        asset.write_bool(self.value)?;
        Ok(())
    }
}

parse_simple_property_prop!(
    FArrayProperty,
    /// Inner property
    inner
);
parse_simple_property_prop!(
    FSetProperty,
    /// Set element
    element_prop
);
parse_simple_property_prop!(
    FMapProperty,
    /// Key
    key_prop,
    /// Value
    value_prop
);

parse_simple_property_index!(
    FObjectProperty,
    /// Class index
    property_class
);
parse_simple_property_index!(
    FSoftObjectProperty,
    /// Class index
    property_class
);
parse_simple_property_index!(
    FClassProperty,
    /// Class index
    property_class,
    /// Meta-class index
    meta_class
);
parse_simple_property_index!(
    FSoftClassProperty,
    /// Class index
    property_class,
    /// Meta-class index
    meta_class
);
parse_simple_property_index!(
    FDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property_index!(
    FMulticastDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property_index!(
    FMulticastInlineDelegateProperty,
    /// Signature function index
    signature_function
);
parse_simple_property_index!(
    FInterfaceProperty,
    /// Interface class index
    interface_class
);
parse_simple_property_index!(
    FByteProperty,
    /// Enum value index
    enum_value
);
parse_simple_property_index!(
    FStructProperty,
    /// Struct value index
    struct_value
);

parse_simple_property!(FNumericProperty);
