pub mod array_property;
pub mod color_property;
pub mod date_property;
pub mod delegate_property;
pub mod enum_property;
pub mod gameplay_tag_container_property;
pub mod guid_property;
pub mod int_property;
pub mod map_property;
pub mod material_input_property;
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

use crate::properties::date_property::TimeSpanProperty;
use crate::properties::sampler_property::SkeletalMeshAreaWeightedTriangleSampler;
use crate::properties::soft_path_property::{
    SoftAssetPathProperty, SoftClassPathProperty, SoftObjectPathProperty,
};
use crate::unreal_types::{Guid, ToFName};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use std::io::Cursor;
use std::io::{Seek, SeekFrom};

use self::{
    array_property::ArrayProperty,
    color_property::{ColorProperty, LinearColorProperty},
    date_property::DateTimeProperty,
    delegate_property::MulticastDelegateProperty,
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
    object_property::{AssetObjectProperty, ObjectProperty, SoftObjectProperty},
    per_platform_property::{
        PerPlatformBoolProperty, PerPlatformFloatProperty, PerPlatformIntProperty,
    },
    rich_curve_key_property::RichCurveKeyProperty,
    sampler_property::{SkeletalMeshSamplingLODBuiltDataProperty, WeightedRandomSamplerProperty},
    set_property::SetProperty,
    smart_name_property::SmartNameProperty,
    str_property::{NameProperty, StrProperty, TextProperty},
    struct_property::StructProperty,
    unknown_property::UnknownProperty,
    vector_property::{
        BoxProperty, IntPointProperty, QuatProperty, RotatorProperty, Vector2DProperty,
        Vector4Property, VectorProperty,
    },
    view_target_blend_property::ViewTargetBlendParamsProperty,
};
use super::error::Error;
use super::{unreal_types::FName, Asset};

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
    ($self:ident, $asset:ident, $cursor:ident, $include_header:ident) => {
        if $include_header {
            $asset.write_property_guid($cursor, &$self.property_guid)?;
        }
    };
}

#[macro_export]
macro_rules! simple_property_write {
    ($property_name:ident, $write_func:ident, $value_name:ident, $value_type:ty) => {
        impl PropertyTrait for $property_name {
            fn write(
                &self,
                asset: &Asset,
                cursor: &mut Cursor<Vec<u8>>,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, cursor, include_header);
                cursor.$write_func::<LittleEndian>(self.$value_name)?;
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
pub trait PropertyDataTrait {
    fn get_name(&self) -> FName;
    fn get_duplication_index(&self) -> i32;
    fn get_property_guid(&self) -> Option<Guid>;
}

#[enum_dispatch]
pub trait PropertyTrait {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error>;
}

#[derive(Hash, PartialEq, Eq)]
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

impl Clone for Property {
    fn clone(&self) -> Self {
        match self {
            Self::BoolProperty(arg0) => Self::BoolProperty(arg0.clone()),
            Self::UInt16Property(arg0) => Self::UInt16Property(arg0.clone()),
            Self::UInt32Property(arg0) => Self::UInt32Property(arg0.clone()),
            Self::UInt64Property(arg0) => Self::UInt64Property(arg0.clone()),
            Self::FloatProperty(arg0) => Self::FloatProperty(arg0.clone()),
            Self::Int16Property(arg0) => Self::Int16Property(arg0.clone()),
            Self::Int64Property(arg0) => Self::Int64Property(arg0.clone()),
            Self::Int8Property(arg0) => Self::Int8Property(arg0.clone()),
            Self::IntProperty(arg0) => Self::IntProperty(arg0.clone()),
            Self::ByteProperty(arg0) => Self::ByteProperty(arg0.clone()),
            Self::DoubleProperty(arg0) => Self::DoubleProperty(arg0.clone()),
            Self::NameProperty(arg0) => Self::NameProperty(arg0.clone()),
            Self::StrProperty(arg0) => Self::StrProperty(arg0.clone()),
            Self::TextProperty(arg0) => Self::TextProperty(arg0.clone()),
            Self::ObjectProperty(arg0) => Self::ObjectProperty(arg0.clone()),
            Self::AssetObjectProperty(arg0) => Self::AssetObjectProperty(arg0.clone()),
            Self::SoftObjectProperty(arg0) => Self::SoftObjectProperty(arg0.clone()),
            Self::IntPointProperty(arg0) => Self::IntPointProperty(arg0.clone()),
            Self::VectorProperty(arg0) => Self::VectorProperty(arg0.clone()),
            Self::Vector4Property(arg0) => Self::Vector4Property(arg0.clone()),
            Self::Vector2DProperty(arg0) => Self::Vector2DProperty(arg0.clone()),
            Self::BoxProperty(arg0) => Self::BoxProperty(arg0.clone()),
            Self::QuatProperty(arg0) => Self::QuatProperty(arg0.clone()),
            Self::RotatorProperty(arg0) => Self::RotatorProperty(arg0.clone()),
            Self::LinearColorProperty(arg0) => Self::LinearColorProperty(arg0.clone()),
            Self::ColorProperty(arg0) => Self::ColorProperty(arg0.clone()),
            Self::TimeSpanProperty(arg0) => Self::TimeSpanProperty(arg0.clone()),
            Self::DateTimeProperty(arg0) => Self::DateTimeProperty(arg0.clone()),
            Self::GuidProperty(arg0) => Self::GuidProperty(arg0.clone()),
            Self::SetProperty(arg0) => Self::SetProperty(arg0.clone()),
            Self::ArrayProperty(arg0) => Self::ArrayProperty(arg0.clone()),
            Self::MapProperty(arg0) => Self::MapProperty(arg0.clone()),
            Self::PerPlatformBoolProperty(arg0) => Self::PerPlatformBoolProperty(arg0.clone()),
            Self::PerPlatformIntProperty(arg0) => Self::PerPlatformIntProperty(arg0.clone()),
            Self::PerPlatformFloatProperty(arg0) => Self::PerPlatformFloatProperty(arg0.clone()),
            Self::MaterialAttributesInputProperty(arg0) => {
                Self::MaterialAttributesInputProperty(arg0.clone())
            }
            Self::ExpressionInputProperty(arg0) => Self::ExpressionInputProperty(arg0.clone()),
            Self::ColorMaterialInputProperty(arg0) => {
                Self::ColorMaterialInputProperty(arg0.clone())
            }
            Self::ScalarMaterialInputProperty(arg0) => {
                Self::ScalarMaterialInputProperty(arg0.clone())
            }
            Self::ShadingModelMaterialInputProperty(arg0) => {
                Self::ShadingModelMaterialInputProperty(arg0.clone())
            }
            Self::VectorMaterialInputProperty(arg0) => {
                Self::VectorMaterialInputProperty(arg0.clone())
            }
            Self::Vector2MaterialInputProperty(arg0) => {
                Self::Vector2MaterialInputProperty(arg0.clone())
            }
            Self::WeightedRandomSamplerProperty(arg0) => {
                Self::WeightedRandomSamplerProperty(arg0.clone())
            }
            Self::SkeletalMeshSamplingLODBuiltDataProperty(arg0) => {
                Self::SkeletalMeshSamplingLODBuiltDataProperty(arg0.clone())
            }
            Self::SkeletalMeshAreaWeightedTriangleSampler(arg0) => {
                Self::SkeletalMeshAreaWeightedTriangleSampler(arg0.clone())
            }
            Self::SoftAssetPathProperty(arg0) => Self::SoftAssetPathProperty(arg0.clone()),
            Self::SoftObjectPathProperty(arg0) => Self::SoftObjectPathProperty(arg0.clone()),
            Self::SoftClassPathProperty(arg0) => Self::SoftClassPathProperty(arg0.clone()),
            Self::MulticastDelegateProperty(arg0) => Self::MulticastDelegateProperty(arg0.clone()),
            Self::RichCurveKeyProperty(arg0) => Self::RichCurveKeyProperty(arg0.clone()),
            Self::ViewTargetBlendParamsProperty(arg0) => {
                Self::ViewTargetBlendParamsProperty(arg0.clone())
            }
            Self::GameplayTagContainerProperty(arg0) => {
                Self::GameplayTagContainerProperty(arg0.clone())
            }
            Self::SmartNameProperty(arg0) => Self::SmartNameProperty(arg0.clone()),
            Self::StructProperty(arg0) => Self::StructProperty(arg0.clone()),
            Self::EnumProperty(arg0) => Self::EnumProperty(arg0.clone()),
            Self::UnknownProperty(arg0) => Self::UnknownProperty(arg0.clone()),
        }
    }
}

impl Property {
    pub fn new(asset: &mut Asset, include_header: bool) -> Result<Option<Self>, Error> {
        let _offset = asset.cursor.position();
        let name = asset.read_fname()?;
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?;
        let length = asset.cursor.read_i32::<LittleEndian>()?;
        let duplication_index = asset.cursor.read_i32::<LittleEndian>()?;

        Property::from_type(
            asset,
            &property_type,
            name,
            include_header,
            length as i64,
            0,
            duplication_index,
        )
        .map(|e| Some(e))
    }
    pub fn from_type(
        asset: &mut Asset,
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

            "SetProperty" => SetProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
                asset.engine_version,
            )?
            .into(),
            "ArrayProperty" => ArrayProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
                asset.engine_version,
                true,
            )?
            .into(),
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

            "StructProperty" => StructProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
                asset.engine_version,
            )?
            .into(),
            "EnumProperty" => {
                EnumProperty::new(asset, name, include_header, length, duplication_index)?.into()
            }
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

    pub fn write(
        property: &Property,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        asset.write_fname(cursor, &property.get_name())?;
        asset.write_fname(cursor, &property.to_fname())?;

        let begin = cursor.position();
        cursor.write_i32::<LittleEndian>(0)?; // initial length
        cursor.write_i32::<LittleEndian>(property.get_duplication_index())?;
        let len = property.write(asset, cursor, include_header)?;
        let end = cursor.position();

        cursor.seek(SeekFrom::Start(begin))?;
        cursor.write_i32::<LittleEndian>(len as i32)?;
        cursor.seek(SeekFrom::Start(end))?;
        Ok(begin as usize)
    }

    pub fn has_custom_serialization(name: &String) -> bool {
        CUSTOM_SERIALIZATION.contains(name)
    }
}

impl ToFName for Property {
    fn to_fname(&self) -> FName {
        match self {
            Property::SkeletalMeshSamplingLODBuiltDataProperty(_) => {
                FName::from_slice("SkeletalMeshSamplingLODBuiltData")
            }
            Property::SkeletalMeshAreaWeightedTriangleSampler(_) => {
                FName::from_slice("SkeletalMeshAreaWeightedTriangleSampler")
            }
            Property::SmartNameProperty(_) => FName::from_slice("SmartName"),
            Property::SoftObjectPathProperty(_) => FName::from_slice("SoftObjectPath"),
            Property::WeightedRandomSamplerProperty(_) => {
                FName::from_slice("WeightedRandomSampler")
            }
            Property::SoftClassPathProperty(_) => FName::from_slice("SoftClassPath"),
            Property::ColorProperty(_) => FName::from_slice("Color"),
            Property::ExpressionInputProperty(_) => FName::from_slice("ExpressionInput"),
            Property::MaterialAttributesInputProperty(_) => {
                FName::from_slice("MaterialAttributesInput")
            }
            Property::ColorMaterialInputProperty(_) => FName::from_slice("ColorMaterialInput"),
            Property::ScalarMaterialInputProperty(_) => FName::from_slice("ScalarMaterialInput"),
            Property::ShadingModelMaterialInputProperty(_) => {
                FName::from_slice("ShadingModelMaterialInput")
            }
            Property::VectorMaterialInputProperty(_) => FName::from_slice("VectorMaterialInput"),
            Property::Vector2MaterialInputProperty(_) => FName::from_slice("Vector2MaterialInput"),
            Property::GameplayTagContainerProperty(_) => FName::from_slice("GameplayTagContainer"),
            Property::PerPlatformBoolProperty(_) => FName::from_slice("PerPlatformBool"),
            Property::PerPlatformIntProperty(_) => FName::from_slice("PerPlatformInt"),
            Property::RichCurveKeyProperty(_) => FName::from_slice("RichCurveKey"),
            Property::SoftAssetPathProperty(_) => FName::from_slice("SoftAssetPath"),
            Property::TimeSpanProperty(_) => FName::from_slice("Timespan"),
            Property::DateTimeProperty(_) => FName::from_slice("DateTime"),
            Property::GuidProperty(_) => FName::from_slice("Guid"),
            Property::IntPointProperty(_) => FName::from_slice("IntPoint"),
            Property::LinearColorProperty(_) => FName::from_slice("LinearColor"),
            Property::QuatProperty(_) => FName::from_slice("Quat"),
            Property::RotatorProperty(_) => FName::from_slice("Rotator"),
            Property::StructProperty(_) => FName::from_slice("StructProperty"),
            Property::Vector2DProperty(_) => FName::from_slice("Vector2D"),
            Property::BoxProperty(_) => FName::from_slice("Box"),
            Property::PerPlatformFloatProperty(_) => FName::from_slice("PerPlatformFloat"),
            Property::Vector4Property(_) => FName::from_slice("Vector4"),
            Property::VectorProperty(_) => FName::from_slice("Vector"),
            Property::ViewTargetBlendParamsProperty(_) => {
                FName::from_slice("ViewTargetBlendParams")
            }
            Property::DoubleProperty(_) => FName::from_slice("DoubleProperty"),
            Property::ArrayProperty(_) => FName::from_slice("ArrayProperty"),
            Property::SetProperty(_) => FName::from_slice("SetProperty"),
            Property::BoolProperty(_) => FName::from_slice("BoolProperty"),
            Property::ByteProperty(_) => FName::from_slice("ByteProperty"),
            Property::EnumProperty(_) => FName::from_slice("EnumProperty"),
            Property::FloatProperty(_) => FName::from_slice("FloatProperty"),
            Property::Int16Property(_) => FName::from_slice("Int16Property"),
            Property::Int64Property(_) => FName::from_slice("Int64Property"),
            Property::Int8Property(_) => FName::from_slice("Int8Property"),
            Property::IntProperty(_) => FName::from_slice("IntProperty"),
            Property::MapProperty(_) => FName::from_slice("MapProperty"),
            Property::MulticastDelegateProperty(_) => {
                FName::from_slice("MulticastDelegateProperty")
            }
            Property::NameProperty(_) => FName::from_slice("NameProperty"),
            Property::ObjectProperty(_) => FName::from_slice("ObjectProperty"),
            Property::AssetObjectProperty(_) => FName::from_slice("AssetObjectProperty"),
            Property::SoftObjectProperty(_) => FName::from_slice("SoftObjectProperty"),
            Property::StrProperty(_) => FName::from_slice("StrProperty"),
            Property::TextProperty(_) => FName::from_slice("TextProperty"),
            Property::UInt16Property(_) => FName::from_slice("UInt16Property"),
            Property::UInt32Property(_) => FName::from_slice("UInt32Property"),
            Property::UInt64Property(_) => FName::from_slice("UInt64Property"),
            Property::UnknownProperty(unk) => unk
                .serialized_type
                .as_ref()
                .map(|e| e.clone())
                .unwrap_or(FName::from_slice("Generic")),
        }
    }
}
