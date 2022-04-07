pub mod int_property;
pub mod str_property;
pub mod object_property;
pub mod guid_property;
pub mod date_property;
pub mod color_property;
pub mod vector_property;
pub mod struct_property;
pub mod array_property;
pub mod set_property;
pub mod map_property;
pub mod unknown_property;
pub mod per_platform_property;
pub mod material_input_property;
pub mod enum_property;
pub mod world_tile_property;
pub mod sampler_property;
pub mod soft_path_property;
pub mod delegate_property;
pub mod rich_curve_key_property;
pub mod view_target_blend_property;
pub mod gameplay_tag_container_property;
pub mod smart_name_property;

use std::{io::{Cursor}, collections::HashMap};
use byteorder::{ReadBytesExt, LittleEndian};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use crate::uasset::properties::date_property::TimeSpanProperty;
use crate::uasset::properties::sampler_property::SkeletalMeshAreaWeightedTriangleSampler;
use crate::uasset::properties::soft_path_property::{SoftAssetPathProperty, SoftClassPathProperty, SoftObjectPathProperty};

use self::{unknown_property::UnknownProperty, int_property::{BoolProperty, UInt16Property, UInt32Property, UInt64Property, FloatProperty, Int16Property, Int64Property, Int8Property, IntProperty, ByteProperty, DoubleProperty}, str_property::{NameProperty, StrProperty, TextProperty}, object_property::{ObjectProperty, AssetObjectProperty, SoftObjectProperty}, vector_property::{IntPointProperty, VectorProperty, Vector4Property, Vector2DProperty, QuatProperty, RotatorProperty, BoxProperty}, color_property::{LinearColorProperty, ColorProperty}, date_property::DateTimeProperty, guid_property::GuidProperty, struct_property::StructProperty, set_property::SetProperty, array_property::ArrayProperty, map_property::MapProperty, per_platform_property::{PerPlatformBoolProperty, PerPlatformIntProperty, PerPlatformFloatProperty}, material_input_property::{MaterialAttributesInputProperty, ExpressionInputProperty, ColorMaterialInputProperty, ScalarMaterialInputProperty, ShadingModelMaterialInputProperty, VectorMaterialInputProperty, Vector2MaterialInputProperty}, enum_property::EnumProperty, sampler_property::{WeightedRandomSamplerProperty, SkeletalMeshSamplingLODBuiltDataProperty}, delegate_property::MulticastDelegateProperty, rich_curve_key_property::RichCurveKeyProperty, view_target_blend_property::ViewTargetBlendParamsProperty, gameplay_tag_container_property::GameplayTagContainerProperty, smart_name_property::SmartNameProperty};
use super::error::Error;
use super::{Asset, unreal_types::FName};

#[macro_export]
macro_rules! optional_guid {
    ($asset:ident, $include_header:ident) => {
        match $include_header {
            true => $asset.read_property_guid()?,
            false => None
        }
    };
}

#[macro_export]
macro_rules! optional_guid_write {
    ($asset:ident, $cursor:ident, $include_header:ident) => {
        if $include_header {
            $asset.write_property_guid($cursor, &self.property_guid)?;
        }
    }
}

#[macro_export]
macro_rules! simple_property_write {
    ($property_name:ident, $write_func:ident, $value_name:ident, $value_type:ty) => {
        impl PropertyTrait for $property_name {
            fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
                optional_guid_write(asset, cursor, include_header);
                cursor.$write_func::<LittleEndian>(self.$value_name)?;
                Ok(size_of::<$value_type>())
            }
        }
    }
}

lazy_static! {
    static ref CUSTOM_SERIALIZATION: Vec<String> = Vec::from([
        String::from("SkeletalMeshSamplingLODBuiltData"),
        String::from("SkeletalMeshAreaWeightedTriangleSampler"),
        String::from("SmartName"),
        String::from("SoftObjectPath"),
        String::from("WeightedRandomSampler"),
        String::from("SoftClassPath"),/
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
    ]);
}

#[enum_dispatch]
trait PropertyTrait {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error>;
}

#[derive(Hash, PartialEq, Eq)]
#[enum_dispatch(PropertyTrait)]
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
    MulticastDelegateProperty,
    RichCurveKeyProperty,
    ViewTargetBlendParamsProperty,
    GameplayTagContainerProperty,
    SmartNameProperty,
    StructProperty,
    EnumProperty,
    UnknownProperty,
}

impl Property {
    pub fn new(asset: &mut Asset, include_header: bool) -> Result<Option<Self>, Error> {
        let offset = asset.cursor.position();
        let name = asset.read_fname()?;
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?;
        let length = asset.cursor.read_i32::<LittleEndian>()?;
        let duplication_index = asset.cursor.read_i32::<LittleEndian>()?;

        Property::from_type(asset, &property_type, name, include_header, length as i64, 0).map(|e| Some(e))
    }
    pub fn from_type(asset: &mut Asset, type_name: &FName, name: FName, include_header: bool, length: i64, fallback_length: i64) -> Result<Self, Error> {
        let res = match type_name.content.as_str() {
            "BoolProperty" => BoolProperty::new(asset, name, include_header, length)?.into(),
            "UInt16Property" => UInt16Property::new(asset, name, include_header, length)?.into(),
            "UInt32Property" => UInt32Property::new(asset, name, include_header, length)?.into(),
            "UInt64Property" => UInt64Property::new(asset, name, include_header, length)?.into(),
            "FloatProperty" => FloatProperty::new(asset, name, include_header, length)?.into(),
            "Int16Property" => Int16Property::new(asset, name, include_header, length)?.into(),
            "Int64Property" => Int64Property::new(asset, name, include_header, length)?.into(),
            "Int8Property" => Int8Property::new(asset, name, include_header, length)?.into(),
            "IntProperty" => IntProperty::new(asset, name, include_header, length)?.into(),
            "ByteProperty" => ByteProperty::new(asset, name, include_header, length, fallback_length)?.into(),
            "DoubleProperty" => DoubleProperty::new(asset, name, include_header, length)?.into(),

            "NameProperty" => NameProperty::new(asset, name, include_header)?.into(),
            "StrProperty" => StrProperty::new(asset, name, include_header)?.into(),
            "TextProperty" => TextProperty::new(asset, name, include_header, asset.engine_version)?.into(),

            "ObjectProperty" => ObjectProperty::new(asset, name, include_header)?.into(),
            "AssetObjectProperty" => AssetObjectProperty::new(asset, name, include_header)?.into(),
            "SoftObjectProperty" => SoftObjectProperty::new(asset, name, include_header)?.into(),

            "IntPoint" => IntPointProperty::new(asset, name, include_header)?.into(),
            "Vector" => VectorProperty::new(asset, name, include_header)?.into(),
            "Vector4" => Vector4Property::new(asset, name, include_header)?.into(),
            "Vector2D" => Vector2DProperty::new(asset, name, include_header)?.into(),
            "Box" => BoxProperty::new(asset, name, include_header)?.into(),
            "Quat" => QuatProperty::new(asset, name, include_header)?.into(),
            "Rotator" => RotatorProperty::new(asset, name, include_header)?.into(),
            "LinearColor" => LinearColorProperty::new(asset, name, include_header)?.into(),
            "Color" => ColorProperty::new(asset, name, include_header)?.into(),
            "Timespan" => TimeSpanProperty::new(asset, name, include_header)?.into(),
            "DateTime" => DateTimeProperty::new(asset, name, include_header)?.into(),
            "Guid" => GuidProperty::new(asset, name, include_header)?.into(),

            "SetProperty" => SetProperty::new(asset, name, include_header, length, asset.engine_version)?.into(),
            "ArrayProperty" => ArrayProperty::new(asset, name, include_header, length, asset.engine_version, true)?.into(),
            "MapProperty" => MapProperty::new(asset, name, include_header)?.into(),

            "PerPlatformBool" => PerPlatformBoolProperty::new(asset, name, include_header, length)?.into(),
            "PerPlatformInt" => PerPlatformIntProperty::new(asset, name, include_header, length)?.into(),
            "PerPlatformFloat" => PerPlatformFloatProperty::new(asset, name, include_header, length)?.into(),

            "MaterialAttributesInput" => MaterialAttributesInputProperty::new(asset, name, include_header)?.into(),
            "ExpressionInput" => ExpressionInputProperty::new(asset, name, include_header)?.into(),
            "ColorMaterialInput" => ColorMaterialInputProperty::new(asset, name, include_header)?.into(),
            "ScalarMaterialInput" => ScalarMaterialInputProperty::new(asset, name, include_header)?.into(),
            "ShadingModelMaterialInput" => ShadingModelMaterialInputProperty::new(asset, name, include_header)?.into(),
            "VectorMaterialInput" => VectorMaterialInputProperty::new(asset, name, include_header)?.into(),
            "Vector2MaterialInput" => Vector2MaterialInputProperty::new(asset, name, include_header)?.into(),

            "WeightedRandomSampler" => WeightedRandomSamplerProperty::new(asset, name, include_header, length)?.into(),
            "SkeletalMeshAreaWeightedTriangleSampler" => SkeletalMeshAreaWeightedTriangleSampler::new(asset, name, include_header, length)?.into(),
            "SkeletalMeshSamplingLODBuiltData" => SkeletalMeshSamplingLODBuiltDataProperty::new(asset, name, include_header, length)?.into(),

            "SoftAssetPath" => SoftAssetPathProperty::new(asset, name, include_header, length)?.into(),
            "SoftObjectPath" => SoftObjectPathProperty::new(asset, name, include_header, length)?.into(),
            "SoftClassPath" => SoftClassPathProperty::new(asset, name, include_header, length)?.into(),

            "MulticastDelegateProperty" => MulticastDelegateProperty::new(asset, name, include_header, length)?.into(),
            "RichCurveKey" => RichCurveKeyProperty::new(asset, name, include_header, length)?.into(),
            "ViewTargetBlendParams" => ViewTargetBlendParamsProperty::new(asset, name, include_header, length)?.into(),
            "GameplayTagContainer" => GameplayTagContainerProperty::new(asset, name, include_header, length)?.into(),
            "SmartName" => SmartNameProperty::new(asset, name, include_header, length)?.into(),

            "StructProperty" => StructProperty::new(asset, name, include_header, length, asset.engine_version)?.into(),
            "EnumProperty" => EnumProperty::new(asset, name, include_header, length)?.into(),
            _ => UnknownProperty::new(asset, name, include_header, length)?.into()
        };
        
        Ok(res)
    }

    pub fn has_custom_serialization(name: &String) -> bool {
        CUSTOM_SERIALIZATION.contains(name)
    }
}

impl ToString for Property {
    fn to_string(&self) -> String {
        match *self {
            Property::SkeletalMeshSamplingLODBuiltDataProperty => "SkeletalMeshSamplingLODBuiltData".to_string(),
            Property::SkeletalMeshAreaWeightedTriangleSampler => "SkeletalMeshAreaWeightedTriangleSampler".to_string(),
            Property::SmartNameProperty => "SmartName".to_string(),
            Property::SoftObjectPathProperty => "SoftObjectPath".to_string(),
            Property::WeightedRandomSamplerProperty => "WeightedRandomSampler".to_string(),
            Property::SoftClassPathProperty => "SoftClassPath".to_string(),
            Property::ColorProperty => "Color".to_string(),
            Property::ExpressionInputProperty => "ExpressionInput".to_string(),
            Property::MaterialAttributesInputProperty => "MaterialAttributesInput".to_string(),
            Property::ColorMaterialInputProperty => "ColorMaterialInput".to_string(),
            Property::ScalarMaterialInputProperty => "ScalarMaterialInput".to_string(),
            Property::ShadingModelMaterialInputProperty => "ShadingModelMaterialInput".to_string(),
            Property::VectorMaterialInputProperty => "VectorMaterialInput".to_string(),
            Property::Vector2MaterialInputProperty => "Vector2MaterialInput".to_string(),
            Property::GameplayTagContainerProperty => "GameplayTagContainer".to_string(),
            Property::PerPlatformBoolProperty => "PerPlatformBool".to_string(),
            Property::PerPlatformIntProperty => "PerPlatformInt".to_string(),
            Property::RichCurveKeyProperty => "RichCurveKey".to_string(),
            Property::SoftAssetPathProperty => "SoftAssetPath".to_string(),
            Property::TimeSpanProperty => "Timespan".to_string(),
            Property::DateTimeProperty => "DateTime".to_string(),
            Property::GuidProperty => "Guid".to_string(),
            Property::IntPointProperty => "IntPoint".to_string(),
            Property::LinearColorProperty => "LinearColor".to_string(),
            Property::QuatProperty => "Quat".to_string(),
            Property::RotatorProperty => "Rotator".to_string(),
            Property::StructProperty => "StructProperty".to_string(),
            Property::Vector2DProperty => "Vector2D".to_string(),
            Property::BoxProperty => "Box".to_string(),
            Property::PerPlatformFloatProperty => "PerPlatformFloat".to_string(),
            Property::Vector4Property => "Vector4".to_string(),
            Property::VectorProperty => "Vector".to_string(),
            Property::ViewTargetBlendParamsProperty => "ViewTargetBlendParams".to_string(),
            Property::DoubleProperty => "DoubleProperty".to_string(),
            Property::ArrayProperty => "ArrayProperty".to_string(),
            Property::SetProperty => "SetProperty".to_string(),
            Property::BoolProperty => "BoolProperty".to_string(),
            Property::ByteProperty => "ByteProperty".to_string(),
            Property::UnknownProperty => "UnknownProperty".to_string(),
            Property::EnumProperty => "EnumProperty".to_string(),
            Property::FloatProperty => "FloatProperty".to_string(),
            Property::Int16Property => "Int16Property".to_string(),
            Property::Int64Property => "Int64Property".to_string(),
            Property::Int8Property => "Int8Property".to_string(),
            Property::IntProperty => "IntProperty".to_string(),
            Property::MapProperty => "MapProperty".to_string(),
            Property::MulticastDelegateProperty => "MulticastDelegateProperty".to_string(),
            Property::NameProperty => "NameProperty".to_string(),
            Property::ObjectProperty => "ObjectProperty".to_string(),
            Property::AssetObjectProperty => "AssetObjectProperty".to_string(),
            Property::SoftObjectProperty => "SoftObjectProperty".to_string(),
            Property::StrProperty => "StrProperty".to_string(),
            Property::TextProperty => "TextProperty".to_string(),
            Property::UInt16Property => "UInt16Property".to_string(),
            Property::UInt32Property => "UInt32Property".to_string(),
            Property::UInt64Property => "UInt64Property".to_string(),
        }
    }
}