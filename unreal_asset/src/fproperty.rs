use byteorder::LittleEndian;
use enum_dispatch::enum_dispatch;

use crate::enums::{EArrayDim, ELifetimeCondition};
use crate::error::Error;
use crate::flags::{EObjectFlags, EPropertyFlags};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, PackageIndex, ToFName};

macro_rules! parse_simple_property {
    ($prop_name:ident) => {
        #[derive(Clone)]
        pub struct $prop_name {
            pub generic_property: FGenericProperty,
        }

        impl $prop_name {
            pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                Ok(())
            }
        }
    };
}

macro_rules! parse_simple_property_index {
    ($prop_name:ident, $($index_name:ident),*) => {
        #[derive(Clone)]
        pub struct $prop_name {
            pub generic_property: FGenericProperty,
            $(
                pub $index_name: PackageIndex,
            )*
        }

        impl $prop_name {
            pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                    $(
                        $index_name: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
                    )*
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                $(
                    asset.write_i32::<LittleEndian>(self.$index_name.index)?;
                )*
                Ok(())
            }
        }
    };
}

macro_rules! parse_simple_property_prop {
    ($prop_name:ident, $($prop:ident),*) => {
        #[derive(Clone)]
        pub struct $prop_name {
            pub generic_property: FGenericProperty,
            $(
                pub $prop: Box<FProperty>,
            )*
        }

        impl $prop_name {
            pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($prop_name {
                    generic_property: FGenericProperty::new(asset)?,
                    $(
                        $prop: Box::new(FProperty::new(asset)?),
                    )*
                })
            }
        }

        impl FPropertyTrait for $prop_name {
            fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
                self.generic_property.write(asset)?;
                $(
                    FProperty::write(self.$prop.as_ref(), asset)?;
                )*
                Ok(())
            }
        }
    };
}

#[enum_dispatch]
pub trait FPropertyTrait {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error>;
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
    FNumericProperty,
}

impl Clone for FProperty {
    fn clone(&self) -> Self {
        match self {
            Self::FGenericProperty(arg0) => Self::FGenericProperty(arg0.clone()),
            Self::FEnumProperty(arg0) => Self::FEnumProperty(arg0.clone()),
            Self::FArrayProperty(arg0) => Self::FArrayProperty(arg0.clone()),
            Self::FSetProperty(arg0) => Self::FSetProperty(arg0.clone()),
            Self::FObjectProperty(arg0) => Self::FObjectProperty(arg0.clone()),
            Self::FSoftObjectProperty(arg0) => Self::FSoftObjectProperty(arg0.clone()),
            Self::FClassProperty(arg0) => Self::FClassProperty(arg0.clone()),
            Self::FSoftClassProperty(arg0) => Self::FSoftClassProperty(arg0.clone()),
            Self::FDelegateProperty(arg0) => Self::FDelegateProperty(arg0.clone()),
            Self::FMulticastDelegateProperty(arg0) => {
                Self::FMulticastDelegateProperty(arg0.clone())
            }
            Self::FMulticastInlineDelegateProperty(arg0) => {
                Self::FMulticastInlineDelegateProperty(arg0.clone())
            }
            Self::FInterfaceProperty(arg0) => Self::FInterfaceProperty(arg0.clone()),
            Self::FMapProperty(arg0) => Self::FMapProperty(arg0.clone()),
            Self::FBoolProperty(arg0) => Self::FBoolProperty(arg0.clone()),
            Self::FByteProperty(arg0) => Self::FByteProperty(arg0.clone()),
            Self::FStructProperty(arg0) => Self::FStructProperty(arg0.clone()),
            Self::FNumericProperty(arg0) => Self::FNumericProperty(arg0.clone()),
        }
    }
}

impl FProperty {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
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
            "MulticastInlineDelegateProperty" => {
                FMulticastInlineDelegateProperty::new(asset)?.into()
            }
            "InterfaceProperty" => FInterfaceProperty::new(asset)?.into(),
            "MapProperty" => FMapProperty::new(asset)?.into(),
            "BoolProperty" => FBoolProperty::new(asset)?.into(),
            "ByteProperty" => FByteProperty::new(asset)?.into(),
            "StructProperty" => FStructProperty::new(asset)?.into(),
            "NumericProperty" => FNumericProperty::new(asset)?.into(),
            _ => {
                FGenericProperty::with_serialized_type(asset, Some(serialized_type.clone()))?.into()
            }
        };

        Ok(res)
    }

    pub fn write<Writer: AssetWriter>(
        property: &FProperty,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_fname(&property.to_fname())?;
        property.write(asset)
    }
}

impl ToFName for FProperty {
    fn to_fname(&self) -> FName {
        match self {
            FProperty::FEnumProperty(_) => FName::from_slice("EnumProperty"),
            FProperty::FArrayProperty(_) => FName::from_slice("ArrayProperty"),
            FProperty::FSetProperty(_) => FName::from_slice("SetProperty"),
            FProperty::FObjectProperty(_) => FName::from_slice("ObjectProperty"),
            FProperty::FSoftObjectProperty(_) => FName::from_slice("SoftObjectProperty"),
            FProperty::FClassProperty(_) => FName::from_slice("ClassProperty"),
            FProperty::FSoftClassProperty(_) => FName::from_slice("SoftClassProperty"),
            FProperty::FDelegateProperty(_) => FName::from_slice("DelegateProperty"),
            FProperty::FMulticastDelegateProperty(_) => {
                FName::from_slice("MulticastDelegateProperty")
            }
            FProperty::FMulticastInlineDelegateProperty(_) => {
                FName::from_slice("MulticastInlineDelegateProperty")
            }
            FProperty::FInterfaceProperty(_) => FName::from_slice("InterfaceProperty"),
            FProperty::FMapProperty(_) => FName::from_slice("MapProperty"),
            FProperty::FBoolProperty(_) => FName::from_slice("BoolProperty"),
            FProperty::FByteProperty(_) => FName::from_slice("ByteProperty"),
            FProperty::FStructProperty(_) => FName::from_slice("StructProperty"),
            FProperty::FNumericProperty(_) => FName::from_slice("NumericProperty"),
            FProperty::FGenericProperty(generic) => generic
                .serialized_type
                .as_ref()
                .cloned()
                .unwrap_or_else(|| FName::from_slice("Generic")),
        }
    }
}

#[derive(Clone)]
pub struct FGenericProperty {
    pub name: FName,
    pub flags: EObjectFlags,
    pub array_dim: EArrayDim,
    pub element_size: i32,
    pub property_flags: EPropertyFlags,
    pub rep_index: u16,
    pub rep_notify_func: FName,
    pub blueprint_replication_condition: ELifetimeCondition,
    pub serialized_type: Option<FName>,
}

#[derive(Clone)]
pub struct FEnumProperty {
    generic_property: FGenericProperty,
    enum_value: PackageIndex,
    underlying_prop: Box<FProperty>,
}

#[derive(Clone)]
pub struct FBoolProperty {
    generic_property: FGenericProperty,

    field_size: u8,
    byte_offset: u8,
    byte_mask: u8,
    field_mask: u8,
    native_bool: bool,
    value: bool,
}

impl FGenericProperty {
    pub fn with_serialized_type<Reader: AssetReader>(
        asset: &mut Reader,
        serialized_type: Option<FName>,
    ) -> Result<Self, Error> {
        let name = asset.read_fname()?;
        let flags: EObjectFlags = EObjectFlags::from_bits(asset.read_u32::<LittleEndian>()?)
            .ok_or_else(|| Error::invalid_file("Invalid object flags".to_string()))?; // todo: maybe other error type than invalid_file?
        let array_dim: EArrayDim = asset.read_i32::<LittleEndian>()?.try_into()?;
        let element_size = asset.read_i32::<LittleEndian>()?;
        let property_flags: EPropertyFlags =
            EPropertyFlags::from_bits(asset.read_u64::<LittleEndian>()?)
                .ok_or_else(|| Error::invalid_file("Invalid property flags".to_string()))?;
        let rep_index = asset.read_u16::<LittleEndian>()?;
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

    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        FGenericProperty::with_serialized_type(asset, None)
    }
}

impl FPropertyTrait for FGenericProperty {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_fname(&self.name)?;
        asset.write_u32::<LittleEndian>(self.flags.bits())?;
        asset.write_i32::<LittleEndian>(self.array_dim.into())?;
        asset.write_i32::<LittleEndian>(self.element_size)?;
        asset.write_u64::<LittleEndian>(self.property_flags.bits())?;
        asset.write_u16::<LittleEndian>(self.rep_index)?;
        asset.write_fname(&self.rep_notify_func)?;
        asset.write_u8(self.blueprint_replication_condition.into())?;
        Ok(())
    }
}

impl FEnumProperty {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let generic_property = FGenericProperty::new(asset)?;
        let enum_value = PackageIndex::new(asset.read_i32::<LittleEndian>()?);
        let underlying_prop = FProperty::new(asset)?;

        Ok(FEnumProperty {
            generic_property,
            enum_value,
            underlying_prop: Box::new(underlying_prop),
        })
    }
}

impl FPropertyTrait for FEnumProperty {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.generic_property.write(asset)?;
        asset.write_i32::<LittleEndian>(self.enum_value.index)?;
        FProperty::write(self.underlying_prop.as_ref(), asset)?;
        Ok(())
    }
}

impl FBoolProperty {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
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
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
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

parse_simple_property_prop!(FArrayProperty, inner);
parse_simple_property_prop!(FSetProperty, element_prop);
parse_simple_property_index!(FObjectProperty, property_class);
parse_simple_property_index!(FSoftObjectProperty, property_class);
parse_simple_property_index!(FClassProperty, property_class, meta_class);
parse_simple_property_index!(FSoftClassProperty, property_class, meta_class);
parse_simple_property_index!(FDelegateProperty, signature_function);
parse_simple_property_index!(FMulticastDelegateProperty, signature_function);
parse_simple_property_index!(FMulticastInlineDelegateProperty, signature_function);
parse_simple_property_index!(FInterfaceProperty, interface_class);
parse_simple_property_prop!(FMapProperty, key_prop, value_prop);
parse_simple_property_index!(FByteProperty, enum_value);
parse_simple_property_index!(FStructProperty, struct_value);
parse_simple_property!(FNumericProperty);
