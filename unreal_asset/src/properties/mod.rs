//! All UAsset properties
use std::hash::Hash;
use std::io::SeekFrom;

use byteorder::LittleEndian;
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;

use crate::error::Error;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid, ToFName};

pub mod array_property;
pub mod cloth_lod_property;
pub mod color_property;
pub mod date_property;
pub mod delegate_property;
pub mod enum_property;
pub mod font_character_property;
pub mod game_framework;
pub mod gameplay_tag_container_property;
pub mod guid_property;
pub mod int_property;
pub mod map_property;
pub mod material_input_property;
pub mod multicast_delegate_property;
pub mod niagara;
pub mod object_property;
pub mod per_platform_property;
pub mod rich_curve_key_property;
pub mod sampler_property;
pub mod set_property;
pub mod smart_name_property;
pub mod soft_path_property;
pub mod str_property;
pub mod struct_property;
pub mod unknown_property;
pub mod vector_property;
pub mod view_target_blend_property;
pub mod world_tile_property;

use self::cloth_lod_property::ClothLodDataProperty;
use self::delegate_property::DelegateProperty;
use self::font_character_property::FontCharacterProperty;
use self::game_framework::unique_net_id_property::UniqueNetIdProperty;
use self::niagara::niagara_variable_property::{
    NiagaraVariableProperty, NiagaraVariableWithOffsetProperty,
};
use self::soft_path_property::StringAssetReferenceProperty;
use self::vector_property::Box2DProperty;
use self::{
    array_property::ArrayProperty,
    color_property::{ColorProperty, LinearColorProperty},
    date_property::{DateTimeProperty, TimeSpanProperty},
    enum_property::EnumProperty,
    gameplay_tag_container_property::GameplayTagContainerProperty,
    guid_property::GuidProperty,
    int_property::{
        BoolProperty, ByteProperty, DoubleProperty, FloatProperty, Int16Property, Int64Property,
        Int8Property, IntProperty, UInt16Property, UInt32Property, UInt64Property,
    },
    map_property::MapProperty,
    material_input_property::{
        ColorMaterialInputProperty, ExpressionInputProperty, MaterialAttributesInputProperty,
        ScalarMaterialInputProperty, ShadingModelMaterialInputProperty,
        Vector2MaterialInputProperty, VectorMaterialInputProperty,
    },
    multicast_delegate_property::MulticastDelegateProperty,
    object_property::{AssetObjectProperty, ObjectProperty, SoftObjectProperty},
    per_platform_property::{
        PerPlatformBoolProperty, PerPlatformFloatProperty, PerPlatformIntProperty,
    },
    rich_curve_key_property::RichCurveKeyProperty,
    sampler_property::{
        SkeletalMeshAreaWeightedTriangleSampler, SkeletalMeshSamplingLODBuiltDataProperty,
        WeightedRandomSamplerProperty,
    },
    set_property::SetProperty,
    smart_name_property::SmartNameProperty,
    soft_path_property::{SoftAssetPathProperty, SoftClassPathProperty, SoftObjectPathProperty},
    str_property::{NameProperty, StrProperty, TextProperty},
    struct_property::StructProperty,
    unknown_property::UnknownProperty,
    vector_property::{
        BoxProperty, IntPointProperty, QuatProperty, RotatorProperty, Vector2DProperty,
        Vector4Property, VectorProperty,
    },
    view_target_blend_property::ViewTargetBlendParamsProperty,
};

#[macro_export]
macro_rules! optional_guid {
    ($asset:ident, $include_header:ident) => {
        match $include_header {
            true => $asset.read_property_guid()?,
            false => None,
        }
    };
}

#[macro_export]
macro_rules! optional_guid_write {
    ($self:ident, $asset:ident, $include_header:ident) => {
        if $include_header {
            $asset.write_property_guid(&$self.property_guid)?;
        }
    };
}

#[macro_export]
macro_rules! simple_property_write {
    ($property_name:ident, $write_func:ident, $value_name:ident, $value_type:ty) => {
        impl PropertyTrait for $property_name {
            fn write<Writer: AssetWriter>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);
                asset.$write_func::<LittleEndian>(self.$value_name)?;
                Ok(size_of::<$value_type>())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_property_data_trait {
    ($property_name:ident) => {
        impl PropertyDataTrait for $property_name {
            fn get_name(&self) -> FName {
                self.name.clone()
            }

            fn get_duplication_index(&self) -> i32 {
                self.duplication_index
            }

            fn get_property_guid(&self) -> Option<Guid> {
                self.property_guid.clone()
            }
        }
    };
}

lazy_static! {
    static ref CUSTOM_SERIALIZATION: Vec<String> = Vec::from([
        String::from("SkeletalMeshSamplingLODBuiltData"),
        String::from("SkeletalMeshAreaWeightedTriangleSampler"),
        String::from("SmartName"),
        String::from("SoftObjectPath"),
        String::from("WeightedRandomSampler"),
        String::from("SoftClassPath"),
        String::from("StringAssetReference"),
        String::from("Color"),
        String::from("ExpressionInput"),
        String::from("MaterialAttributesInput"),
        String::from("ColorMaterialInput"),
        String::from("ScalarMaterialInput"),
        String::from("ShadingModelMaterialInput"),
        String::from("VectorMaterialInput"),
        String::from("Vector2MaterialInput"),
        String::from("GameplayTagContainer"),
        String::from("PerPlatformBool"),
        String::from("PerPlatformInt"),
        String::from("RichCurveKey"),
        String::from("SoftAssetPath"),
        String::from("Timespan"),
        String::from("DateTime"),
        String::from("Guid"),
        String::from("IntPoint"),
        String::from("LinearColor"),
        String::from("Quat"),
        String::from("Rotator"),
        String::from("Vector2D"),
        String::from("Box"),
        String::from("PerPlatformFloat"),
        String::from("Vector4"),
        String::from("Vector"),
        String::from("ViewTargetBlendParams"),
        String::from("FontCharacter"),
        String::from("UniqueNetIdRepl"),
        String::from("NiagaraVariable")
    ]);
}

#[enum_dispatch]
pub trait PropertyDataTrait {
    fn get_name(&self) -> FName;
    fn get_duplication_index(&self) -> i32;
    fn get_property_guid(&self) -> Option<Guid>;
}

/// This must be implemented for all Properties
#[enum_dispatch]
pub trait PropertyTrait {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error>;
}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(PropertyTrait, PropertyDataTrait)]
pub enum Property {
    BoolProperty,
    UInt16Property,
    UInt32Property,
    UInt64Property,
    FloatProperty,
    Int16Property,
    Int64Property,
    Int8Property,
    IntProperty,
    ByteProperty,
    DoubleProperty,
    NameProperty,
    StrProperty,
    TextProperty,
    ObjectProperty,
    AssetObjectProperty,
    SoftObjectProperty,
    IntPointProperty,
    VectorProperty,
    Vector4Property,
    Vector2DProperty,
    BoxProperty,
    Box2DProperty,
    QuatProperty,
    RotatorProperty,
    LinearColorProperty,
    ColorProperty,
    TimeSpanProperty,
    DateTimeProperty,
    GuidProperty,
    SetProperty,
    ArrayProperty,
    MapProperty,
    PerPlatformBoolProperty,
    PerPlatformIntProperty,
    PerPlatformFloatProperty,
    MaterialAttributesInputProperty,
    ExpressionInputProperty,
    ColorMaterialInputProperty,
    ScalarMaterialInputProperty,
    ShadingModelMaterialInputProperty,
    VectorMaterialInputProperty,
    Vector2MaterialInputProperty,
    WeightedRandomSamplerProperty,
    SkeletalMeshSamplingLODBuiltDataProperty,
    SkeletalMeshAreaWeightedTriangleSampler,
    SoftAssetPathProperty,
    SoftObjectPathProperty,
    SoftClassPathProperty,
    StringAssetReferenceProperty,
    DelegateProperty,
    MulticastDelegateProperty,
    RichCurveKeyProperty,
    ViewTargetBlendParamsProperty,
    GameplayTagContainerProperty,
    SmartNameProperty,
    StructProperty,
    EnumProperty,
    ClothLodDataProperty,
    FontCharacterProperty,
    UniqueNetIdProperty,
    NiagaraVariableProperty,
    NiagaraVariableWithOffset,

    UnknownProperty,
}

macro_rules! inner_trait {
    ($outer_name:ty, $($inner:ident),*) => {
        impl Hash for $outer_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                match self {
                    $(
                        Self::$inner(inner) => inner.hash(state),
                    )*
                }
            }
        }

        impl PartialEq for $outer_name {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    $(
                        (Self::$inner(l0), Self::$inner(r0)) => l0 == r0,
                    )*
                    _ => false
                }
            }
        }

        impl Clone for $outer_name {
            fn clone(&self) -> Self {
                match self {
                    $(
                        Self::$inner(arg0) => Self::$inner(arg0.clone()),
                    )*
                }
            }
        }
    };
}

inner_trait!(
    Property,
    BoolProperty,
    UInt16Property,
    UInt32Property,
    UInt64Property,
    FloatProperty,
    Int16Property,
    Int64Property,
    Int8Property,
    IntProperty,
    ByteProperty,
    DoubleProperty,
    NameProperty,
    StrProperty,
    TextProperty,
    ObjectProperty,
    AssetObjectProperty,
    SoftObjectProperty,
    IntPointProperty,
    VectorProperty,
    Vector4Property,
    Vector2DProperty,
    BoxProperty,
    Box2DProperty,
    QuatProperty,
    RotatorProperty,
    LinearColorProperty,
    ColorProperty,
    TimeSpanProperty,
    DateTimeProperty,
    GuidProperty,
    SetProperty,
    ArrayProperty,
    MapProperty,
    PerPlatformBoolProperty,
    PerPlatformIntProperty,
    PerPlatformFloatProperty,
    MaterialAttributesInputProperty,
    ExpressionInputProperty,
    ColorMaterialInputProperty,
    ScalarMaterialInputProperty,
    ShadingModelMaterialInputProperty,
    VectorMaterialInputProperty,
    Vector2MaterialInputProperty,
    WeightedRandomSamplerProperty,
    SkeletalMeshSamplingLODBuiltDataProperty,
    SkeletalMeshAreaWeightedTriangleSampler,
    SoftAssetPathProperty,
    SoftObjectPathProperty,
    SoftClassPathProperty,
    StringAssetReferenceProperty,
    DelegateProperty,
    MulticastDelegateProperty,
    RichCurveKeyProperty,
    ViewTargetBlendParamsProperty,
    GameplayTagContainerProperty,
    SmartNameProperty,
    StructProperty,
    EnumProperty,
    ClothLodDataProperty,
    FontCharacterProperty,
    UniqueNetIdProperty,
    NiagaraVariableProperty,
    NiagaraVariableWithOffsetProperty,
    UnknownProperty
);

impl Eq for Property {}

impl Property {
    /// Tries to read a property from an AssetReader
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        include_header: bool,
    ) -> Result<Option<Self>, Error> {
        let name = asset.read_fname()?;
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?;
        let length = asset.read_i32::<LittleEndian>()?;
        let duplication_index = asset.read_i32::<LittleEndian>()?;

        Property::from_type(
            asset,
            &property_type,
            name,
            include_header,
            length as i64,
            0,
            duplication_index,
        )
        .map(Some)
    }

    /// Tries to read a property from an AssetReader while specified a type and length
    pub fn from_type<Reader: AssetReader>(
        asset: &mut Reader,
        type_name: &FName,
        name: FName,
        include_header: bool,
        length: i64,
        fallback_length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let res = match type_name.content.as_str() {
            "BoolProperty" => {
                BoolProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "UInt16Property" => {
                UInt16Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "UInt32Property" => {
                UInt32Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "UInt64Property" => {
                UInt64Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "FloatProperty" => {
                FloatProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "Int16Property" => {
                Int16Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "Int64Property" => {
                Int64Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "Int8Property" => {
                Int8Property::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "IntProperty" => {
                IntProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "ByteProperty" => ByteProperty::new(
                asset,
                name,
                include_header,
                length,
                fallback_length,
                duplication_index,
            )?
            .into(),
            "DoubleProperty" => {
                DoubleProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }

            "NameProperty" => {
                NameProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "StrProperty" => {
                StrProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "TextProperty" => {
                TextProperty::new(asset, name, include_header, duplication_index)?.into()
            }

            "ObjectProperty" => {
                ObjectProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "AssetObjectProperty" => {
                AssetObjectProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "SoftObjectProperty" => {
                SoftObjectProperty::new(asset, name, include_header, duplication_index)?.into()
            }

            "IntPoint" => {
                IntPointProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "Vector" => VectorProperty::new(asset, name, include_header, duplication_index)?.into(),
            "Vector4" => {
                Vector4Property::new(asset, name, include_header, duplication_index)?.into()
            }
            "Vector2D" => {
                Vector2DProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "Box" => BoxProperty::new(asset, name, include_header, duplication_index)?.into(),
            "Box2D" => Box2DProperty::new(asset, name, include_header, duplication_index)?.into(),
            "Quat" => QuatProperty::new(asset, name, include_header, duplication_index)?.into(),
            "Rotator" => {
                RotatorProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "LinearColor" => {
                LinearColorProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "Color" => ColorProperty::new(asset, name, include_header, duplication_index)?.into(),
            "Timespan" => {
                TimeSpanProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "DateTime" => {
                DateTimeProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "Guid" => GuidProperty::new(asset, name, include_header, duplication_index)?.into(),

            "SetProperty" => {
                SetProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "ArrayProperty" => {
                ArrayProperty::new(asset, name, include_header, length, duplication_index, true)?
                    .into()
            }
            "MapProperty" => {
                MapProperty::new(asset, name, include_header, duplication_index)?.into()
            }

            "PerPlatformBool" => PerPlatformBoolProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "PerPlatformInt" => {
                PerPlatformIntProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "PerPlatformFloat" => PerPlatformFloatProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "MaterialAttributesInput" => MaterialAttributesInputProperty::new(
                asset,
                name,
                include_header,
                duplication_index,
            )?
            .into(),
            "ExpressionInput" => {
                ExpressionInputProperty::new(asset, name, include_header, duplication_index)?.into()
            }
            "ColorMaterialInput" => {
                ColorMaterialInputProperty::new(asset, name, include_header, duplication_index)?
                    .into()
            }
            "ScalarMaterialInput" => {
                ScalarMaterialInputProperty::new(asset, name, include_header, duplication_index)?
                    .into()
            }
            "ShadingModelMaterialInput" => ShadingModelMaterialInputProperty::new(
                asset,
                name,
                include_header,
                duplication_index,
            )?
            .into(),
            "VectorMaterialInput" => {
                VectorMaterialInputProperty::new(asset, name, include_header, duplication_index)?
                    .into()
            }
            "Vector2MaterialInput" => {
                Vector2MaterialInputProperty::new(asset, name, include_header, duplication_index)?
                    .into()
            }

            "WeightedRandomSampler" => WeightedRandomSamplerProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SkeletalMeshAreaWeightedTriangleSampler" => {
                SkeletalMeshAreaWeightedTriangleSampler::new(
                    asset,
                    name,
                    include_header,
                    length,
                    duplication_index,
                )?
                .into()
            }
            "SkeletalMeshSamplingLODBuiltData" => SkeletalMeshSamplingLODBuiltDataProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "SoftAssetPath" => {
                SoftAssetPathProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "SoftObjectPath" => {
                SoftObjectPathProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "SoftClassPath" => {
                SoftClassPathProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "StringAssetReference" => StringAssetReferenceProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "DelegateProperty" => {
                DelegateProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "MulticastDelegateProperty" => MulticastDelegateProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "RichCurveKey" => {
                RichCurveKeyProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "ViewTargetBlendParams" => ViewTargetBlendParamsProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "GameplayTagContainer" => GameplayTagContainerProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SmartName" => {
                SmartNameProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }

            "StructProperty" => {
                StructProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "EnumProperty" => {
                EnumProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
            "ClothLodDataProperty" => {
                ClothLodDataProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }

            "FontCharacter" => {
                FontCharacterProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "UniqueNetIdRepl" => {
                UniqueNetIdProperty::new(asset, name, include_header, length, duplication_index)?
                    .into()
            }
            "NiagaraVariable" => NiagaraVariableProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "NiagaraVariableWithOffset" => NiagaraVariableWithOffsetProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            _ => UnknownProperty::with_serialized_type(
                asset,
                name,
                include_header,
                length,
                duplication_index,
                Some(type_name.clone()),
            )?
            .into(),
        };

        Ok(res)
    }

    /// Writes a property to an AssetWriter
    pub fn write<Writer: AssetWriter>(
        property: &Property,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        asset.write_fname(&property.get_name())?;
        asset.write_fname(&property.to_fname())?;

        let begin = asset.position();
        asset.write_i32::<LittleEndian>(0)?; // initial length
        asset.write_i32::<LittleEndian>(property.get_duplication_index())?;
        let len = property.write(asset, include_header)?;
        let end = asset.position();

        asset.seek(SeekFrom::Start(begin))?;
        asset.write_i32::<LittleEndian>(len as i32)?;
        asset.seek(SeekFrom::Start(end))?;
        Ok(begin as usize)
    }

    /// Check if a property type has custom serialization
    pub fn has_custom_serialization(name: &String) -> bool {
        CUSTOM_SERIALIZATION.contains(name)
    }
}

macro_rules! property_inner_fname {
    ($($inner:ident : $name:expr),*) => {
        impl ToFName for Property {
            fn to_fname(&self) -> FName {
                match self {
                    $(
                        Self::$inner(_) => FName::from_slice($name),
                    )*
                    Self::UnknownProperty(unk) => unk
                        .serialized_type
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| FName::from_slice("Generic")),
                }
            }
        }
    };
}

property_inner_fname! {
    SkeletalMeshSamplingLODBuiltDataProperty: "SkeletalMeshSamplingLODBuiltData",
    SkeletalMeshAreaWeightedTriangleSampler: "SkeletalMeshAreaWeightedTriangleSampler",
    SmartNameProperty: "SmartName",
    SoftObjectPathProperty: "SoftObjectPath",
    WeightedRandomSamplerProperty: "WeightedRandomSampler",
    SoftClassPathProperty: "SoftClassPath",
    StringAssetReferenceProperty: "StringAssetReference",
    ColorProperty: "Color",
    ExpressionInputProperty: "ExpressionInput",
    MaterialAttributesInputProperty: "MaterialAttributesInput",
    ColorMaterialInputProperty: "ColorMaterialInput",
    ScalarMaterialInputProperty: "ScalarMaterialInput",
    ShadingModelMaterialInputProperty: "ShadingModelMaterialInput",
    VectorMaterialInputProperty: "VectorMaterialInput",
    Vector2MaterialInputProperty: "Vector2MaterialInput",
    GameplayTagContainerProperty: "GameplayTagContainer",
    PerPlatformBoolProperty: "PerPlatformBool",
    PerPlatformIntProperty: "PerPlatformInt",
    RichCurveKeyProperty: "RichCurveKey",
    SoftAssetPathProperty: "SoftAssetPath",
    TimeSpanProperty: "Timespan",
    DateTimeProperty: "DateTime",
    GuidProperty: "Guid",
    IntPointProperty: "IntPoint",
    LinearColorProperty: "LinearColor",
    QuatProperty: "Quat",
    RotatorProperty: "Rotator",
    StructProperty: "StructProperty",
    Vector2DProperty: "Vector2D",
    BoxProperty: "Box",
    Box2DProperty: "Box2D",
    PerPlatformFloatProperty: "PerPlatformFloat",
    Vector4Property: "Vector4",
    VectorProperty: "Vector",
    ViewTargetBlendParamsProperty: "ViewTargetBlendParams",
    DoubleProperty: "DoubleProperty",
    ArrayProperty: "ArrayProperty",
    SetProperty: "SetProperty",
    BoolProperty: "BoolProperty",
    ByteProperty: "ByteProperty",
    EnumProperty: "EnumProperty",
    ClothLodDataProperty: "ClothLodData",
    FloatProperty: "FloatProperty",
    Int16Property: "Int16Property",
    Int64Property: "Int64Property",
    Int8Property: "Int8Property",
    IntProperty: "IntProperty",
    MapProperty: "MapProperty",
    MulticastDelegateProperty: "MulticastDelegateProperty",
    DelegateProperty: "DelegateProperty",
    NameProperty: "NameProperty",
    ObjectProperty: "ObjectProperty",
    AssetObjectProperty: "AssetObjectProperty",
    SoftObjectProperty: "SoftObjectProperty",
    StrProperty: "StrProperty",
    TextProperty: "TextProperty",
    UInt16Property: "UInt16Property",
    UInt32Property: "UInt32Property",
    UInt64Property: "UInt64Property"
}
