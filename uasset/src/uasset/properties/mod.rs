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

use std::{io::{Error, Cursor}, collections::HashMap};
use byteorder::{ReadBytesExt, LittleEndian};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;

use self::{unknown_property::UnknownProperty, int_property::{BoolProperty, UInt16Property, UInt32Property, UInt64Property, FloatProperty, Int16Property, Int64Property, Int8Property, IntProperty, ByteProperty, DoubleProperty}, str_property::{NameProperty, StrProperty, TextProperty}, object_property::{ObjectProperty, AssetObjectProperty, SoftObjectProperty}, vector_property::{IntPointProperty, VectorProperty, Vector4Property, Vector2DProperty, QuatProperty, RotatorProperty, BoxProperty}, color_property::{LinearColorProperty, ColorProperty}, date_property::DateTimeProperty, guid_property::GuidProperty, struct_property::StructProperty, set_property::SetProperty, array_property::ArrayProperty, map_property::MapProperty, per_platform_property::{PerPlatformBoolProperty, PerPlatformIntProperty, PerPlatformFloatProperty}, material_input_property::{MaterialAttributesInputProperty, ExpressionInputProperty, ColorMaterialInputProperty, ScalarMaterialInputProperty, ShadingModelMaterialInputProperty, VectorMaterialInputProperty, Vector2MaterialInputProperty}, enum_property::EnumProperty, sampler_property::{WeightedRandomSamplerProperty, SkeletalMeshSamplingLODBuiltDataProperty}, soft_path_property::SoftPathProperty, delegate_property::MulticastDelegateProperty, rich_curve_key_property::RichCurveKeyProperty, view_target_blend_property::ViewTargetBlendParamsProperty, gameplay_tag_container_property::GameplayTagContainerProperty, smart_name_property::SmartNameProperty};

use super::{Asset, unreal_types::FName};

#[macro_export]
macro_rules! optional_guid {
    ($cursor:ident, $include_header:ident) => {
        match $include_header {
            true => Some($cursor.read_property_guid()?),
            false => None
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
    SoftPathProperty,
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
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &Asset, include_header: bool) -> Result<Option<Self>, Error> {
        let offset = cursor.position();
        let name = asset.read_fname()?; // probably should pass cursor instancce there
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?; // probably should pass cursor instance there
        let length = cursor.read_i32::<LittleEndian>()?;
        let duplication_index = cursor.read_i32::<LittleEndian>()?;

        Property::from_type(cursor, asset, property_type, name, include_header, length as i64, 0).map(|e| Some(e))
    }

    pub fn from_type(cursor: &mut Cursor<Vec<u8>>, asset: &Asset, type_name: FName, name: FName, include_header: bool, length: i64, fallback_length: i64) -> Result<Self, Error> {
        let res = match type_name.content.as_str() {
            "BoolProperty" => Property::BoolProperty(BoolProperty::new(name, cursor, include_header, length)?),
            "UInt16Property" => Property::UInt16Property(UInt16Property::new(name, cursor, include_header, length)?),
            "UInt32Property" => Property::UInt32Property(UInt32Property::new(name, cursor, include_header, length)?),
            "UInt64Property" => Property::UInt64Property(UInt64Property::new(name, cursor, include_header, length)?),
            "FloatProperty" => Property::FloatProperty(FloatProperty::new(name, cursor, include_header, length)?),
            "Int16Property" => Property::Int16Property(Int16Property::new(name, cursor, include_header, length)?),
            "Int64Property" => Property::Int64Property(Int64Property::new(name, cursor, include_header, length)?),
            "Int8Property" => Property::Int8Property(Int8Property::new(name, cursor, include_header, length)?),
            "IntProperty" => Property::IntProperty(IntProperty::new(name, cursor, include_header, length)?),
            "ByteProperty" => Property::ByteProperty(ByteProperty::new(name, cursor, include_header, length, fallback_length)?),
            "DoubleProperty" => Property::DoubleProperty(DoubleProperty::new(name, cursor, include_header, length)?),

            "NameProperty" => Property::NameProperty(NameProperty::new(name, cursor, include_header, asset)?),
            "StrProperty" => Property::StrProperty(StrProperty::new(name, cursor, include_header)?),
            "TextProperty" => Property::TextProperty(TextProperty::new(name, cursor, include_header, asset.engine_version, asset)?),

            "ObjectProperty" => Property::ObjectProperty(ObjectProperty::new(name, cursor, include_header)?),
            "AssetObjectProperty" => Property::AssetObjectProperty(AssetObjectProperty::new(name, cursor, include_header)?),
            "SoftObjectProperty" => Property::SoftObjectProperty(SoftObjectProperty::new(name, cursor, include_header, asset)?),

            "IntPoint" => Property::IntPointProperty(IntPointProperty::new(name, cursor, include_header)?),
            "Vector" => Property::VectorProperty(VectorProperty::new(name, cursor, include_header)?),
            "Vector4" => Property::Vector4Property(Vector4Property::new(name, cursor, include_header)?),
            "Vector2D" => Property::Vector2DProperty(Vector2DProperty::new(name, cursor, include_header)?),
            "Box" => Property::BoxProperty(BoxProperty::new(name, cursor, include_header)?),
            "Quat" => Property::QuatProperty(QuatProperty::new(name, cursor, include_header)?),
            "Rotator" => Property::RotatorProperty(RotatorProperty::new(name, cursor, include_header)?),
            "LinearColor" => Property::LinearColorProperty(LinearColorProperty::new(name, cursor, include_header)?),
            "Color" => Property::ColorProperty(ColorProperty::new(name, cursor, include_header)?),
            "Timespan" => Property::DateTimeProperty(DateTimeProperty::new(name, cursor, include_header)?),
            "DateTime" => Property::DateTimeProperty(DateTimeProperty::new(name, cursor, include_header)?),
            "Guid" => Property::GuidProperty(GuidProperty::new(name, cursor, include_header)?),

            "SetProperty" => Property::SetProperty(SetProperty::new(name, cursor, include_header, length, asset.engine_version, asset)?),
            "ArrayProperty" => Property::ArrayProperty(ArrayProperty::new(name, cursor, include_header, length, asset.engine_version, asset, true)?),
            "MapProperty" => Property::MapProperty(MapProperty::new(name, cursor, include_header, asset)?),

            "PerPlatformBool" => Property::PerPlatformBoolProperty(PerPlatformBoolProperty::new(name, cursor, include_header, length)?),
            "PerPlatformInt" => Property::PerPlatformIntProperty(PerPlatformIntProperty::new(name, cursor, include_header, length)?),
            "PerPlatformFloat" => Property::PerPlatformFloatProperty(PerPlatformFloatProperty::new(name, cursor, include_header, length)?),

            "MaterialAttributesInput" => Property::MaterialAttributesInputProperty(MaterialAttributesInputProperty::new(name, cursor, include_header, asset)?),
            "ExpressionInput" => Property::ExpressionInputProperty(ExpressionInputProperty::new(name, cursor, include_header, asset)?),
            "ColorMaterialInput" => Property::ColorMaterialInputProperty(ColorMaterialInputProperty::new(name, cursor, include_header, asset)?),
            "ScalarMaterialInput" => Property::ScalarMaterialInputProperty(ScalarMaterialInputProperty::new(name, cursor, include_header, asset)?),
            "ShadingModelMaterialInput" => Property::ShadingModelMaterialInputProperty(ShadingModelMaterialInputProperty::new(name, cursor, include_header, asset)?),
            "VectorMaterialInput" => Property::VectorMaterialInputProperty(VectorMaterialInputProperty::new(name, cursor, include_header, asset)?),
            "Vector2MaterialInput" => Property::Vector2MaterialInputProperty(Vector2MaterialInputProperty::new(name, cursor, include_header, asset)?),

            "WeightedRandomSampler" => Property::WeightedRandomSamplerProperty(WeightedRandomSamplerProperty::new(name, cursor, include_header, length)?),
            "SkeletalMeshAreaWeightedTriangleSampler" => Property::WeightedRandomSamplerProperty(WeightedRandomSamplerProperty::new(name, cursor, include_header, length)?),
            "SkeletalMeshSamplingLODBuiltData" => Property::SkeletalMeshSamplingLODBuiltDataProperty(SkeletalMeshSamplingLODBuiltDataProperty::new(name, cursor, include_header, length)?),

            "SoftAssetPath" | "SoftObjectPath" | "SoftClassPath" => Property::SoftPathProperty(SoftPathProperty::new(name, cursor, include_header, length, asset)?),

            "MulticastDelegateProperty" => Property::MulticastDelegateProperty(MulticastDelegateProperty::new(name, cursor, include_header, length, asset)?),
            "RichCurveKey" => Property::RichCurveKeyProperty(RichCurveKeyProperty::new(name, cursor, include_header, length)?),
            "ViewTargetBlendParams" => Property::ViewTargetBlendParamsProperty(ViewTargetBlendParamsProperty::new(name, cursor, include_header, length)?),
            "GameplayTagContainer" => Property::GameplayTagContainerProperty(GameplayTagContainerProperty::new(name, cursor, include_header, length, asset)?),
            "SmartName" => Property::SmartNameProperty(SmartNameProperty::new(name, cursor, include_header, length, asset)?),

            "StructProperty" => Property::StructProperty(StructProperty::new(name, cursor, include_header, length, asset.engine_version, asset)?),
            "EnumProperty" => Property::EnumProperty(EnumProperty::new(name, cursor, include_header, length, asset)?),
            _ => Property::UnknownProperty(UnknownProperty::new(name, cursor, include_header, length)?)
        };
        
        Ok(res)
    }

    pub fn has_custom_serialization(name: String) -> bool {
        CUSTOM_SERIALIZATION.contains(&name)
    }
}
