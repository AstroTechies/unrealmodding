//! All UAsset properties
use std::fmt::Debug;
use std::hash::Hash;
use std::io::SeekFrom;

use byteorder::LE;
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::{Error, PropertyError};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{
    fname::{FName, ToSerializedName},
    Guid,
};
use crate::unversioned::ancestry::Ancestry;
use crate::unversioned::header::UnversionedHeader;
use crate::unversioned::properties::UsmapPropertyDataTrait;

pub mod array_property;
pub mod cloth_lod_property;
pub mod color_property;
pub mod date_property;
pub mod delegate_property;
pub mod empty_property;
pub mod enum_property;
pub mod float_range_property;
pub mod font_character_property;
pub mod game_framework;
pub mod gameplay_tag_container_property;
pub mod guid_property;
pub mod int_property;
pub mod map_property;
pub mod material_input_property;
pub mod movies;
pub mod niagara;
pub mod object_property;
pub mod per_platform_property;
pub mod raw_struct_property;
pub mod rich_curve_key_property;
pub mod sampler_property;
pub mod set_property;
pub mod slate_core;
pub mod smart_name_property;
pub mod soft_path_property;
pub mod str_property;
pub mod struct_property;
pub mod unknown_property;
pub mod vector_property;
pub mod view_target_blend_property;
pub mod world_tile_property;

use self::cloth_lod_property::ClothLodDataProperty;
use self::float_range_property::FloatRangeProperty;
use self::font_character_property::FontCharacterProperty;
use self::game_framework::unique_net_id_property::UniqueNetIdProperty;
use self::movies::movie_scene_eval_template_ptr_property::MovieSceneEvalTemplatePtrProperty;
use self::movies::movie_scene_evaluation_field_entity_tree_property::MovieSceneEvaluationFieldEntityTreeProperty;
use self::movies::movie_scene_evaluation_key_property::MovieSceneEvaluationKeyProperty;
use self::movies::movie_scene_event_parameters_property::MovieSceneEventParametersProperty;
use self::movies::movie_scene_float_channel_property::MovieSceneFloatChannelProperty;
use self::movies::movie_scene_float_value_property::MovieSceneFloatValueProperty;
use self::movies::movie_scene_frame_range_property::MovieSceneFrameRangeProperty;
use self::movies::movie_scene_segment_property::{
    MovieSceneSegmentIdentifierProperty, MovieSceneSegmentProperty,
};
use self::movies::movie_scene_sequence_id_property::MovieSceneSequenceIdProperty;
use self::movies::movie_scene_sequence_instance_data_ptr_property::MovieSceneSequenceInstanceDataPtrProperty;
use self::movies::movie_scene_sub_sequence_tree_property::MovieSceneSubSequenceTreeProperty;
use self::movies::movie_scene_track_field_data_property::MovieSceneTrackFieldDataProperty;
use self::movies::movie_scene_track_identifier_property::MovieSceneTrackIdentifierProperty;
use self::movies::movie_scene_track_implementation_ptr_property::MovieSceneTrackImplementationPtrProperty;
use self::movies::section_evaluation_data_tree_property::SectionEvaluationDataTreeProperty;
use self::niagara::niagara_variable_property::{
    NiagaraVariableProperty, NiagaraVariableWithOffsetProperty,
};
use self::raw_struct_property::RawStructProperty;
use self::slate_core::font_data_property::FontDataProperty;
use self::soft_path_property::StringAssetReferenceProperty;
use self::vector_property::Box2DProperty;
use self::{
    array_property::ArrayProperty,
    color_property::{ColorProperty, LinearColorProperty},
    date_property::{DateTimeProperty, TimeSpanProperty},
    delegate_property::{
        DelegateProperty, MulticastDelegateProperty, MulticastInlineDelegateProperty,
        MulticastSparseDelegateProperty,
    },
    empty_property::EmptyProperty,
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

/// Read a property guid if reading with header
#[macro_export]
macro_rules! optional_guid {
    ($asset:ident, $include_header:ident) => {
        match $include_header {
            true => $asset.read_property_guid()?,
            false => None,
        }
    };
}

/// Write a property guid if writing with header
#[macro_export]
macro_rules! optional_guid_write {
    ($self:ident, $asset:ident, $include_header:ident) => {
        if $include_header {
            $asset.write_property_guid(&$self.property_guid)?;
        }
    };
}

/// Write a simple one-value property
#[macro_export]
macro_rules! simple_property_write {
    ($property_name:ident, $write_func:ident, $value_name:ident, $value_type:ty) => {
        impl PropertyTrait for $property_name {
            fn write<Writer: ArchiveWriter>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);
                asset.$write_func::<LE>(self.$value_name)?;
                Ok(size_of::<$value_type>())
            }
        }
    };
}

/// Default implementations for `PropertyDataTrait`
#[macro_export]
macro_rules! impl_property_data_trait {
    ($property_name:ident) => {
        impl $crate::properties::PropertyDataTrait for $property_name {
            fn get_name(&self) -> $crate::types::fname::FName {
                self.name.clone()
            }

            fn get_name_mut(&mut self) -> &mut FName {
                &mut self.name
            }

            fn get_duplication_index(&self) -> i32 {
                self.duplication_index
            }

            fn get_property_guid(&self) -> Option<$crate::types::Guid> {
                self.property_guid.clone()
            }

            fn get_ancestry(&self) -> &$crate::unversioned::ancestry::Ancestry {
                &self.ancestry
            }

            fn get_ancestry_mut(&mut self) -> &mut $crate::unversioned::ancestry::Ancestry {
                &mut self.ancestry
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
        String::from("NiagaraVariable"),
        String::from("FontData"),
        String::from("ClothLODData"),
        String::from("FloatRange"),
        String::from("RawStructProperty"),
        //
        String::from("MovieSceneEvalTemplatePtr"),
        String::from("MovieSceneTrackImplementationPtr"),
        String::from("MovieSceneEvaluationFieldEntityTree"),
        String::from("MovieSceneSubSequenceTree"),
        String::from("MovieSceneSequenceInstanceDataPtr"),
        String::from("SectionEvaluationDataTree"),
        String::from("MovieSceneTrackFieldData"),
        String::from("MovieSceneEventParameters"),
        String::from("MovieSceneFloatChannel"),
        String::from("MovieSceneFloatValue"),
        String::from("MovieSceneFrameRange"),
        String::from("MovieSceneSegment"),
        String::from("MovieSceneSegmentIdentifier"),
        String::from("MovieSceneTrackIdentifier"),
        String::from("MovieSceneSequenceId"),
        String::from("MovieSceneEvaluationKey")
    ]);
}

/// This must be implemented for all properties
#[enum_dispatch]
pub trait PropertyDataTrait {
    /// Get property's name
    fn get_name(&self) -> FName;
    /// Get a mutable reference to property's name
    fn get_name_mut(&mut self) -> &mut FName;
    /// Get property's duplication index
    fn get_duplication_index(&self) -> i32;
    /// Get property's guid
    fn get_property_guid(&self) -> Option<Guid>;
    /// Get property's ancestry
    fn get_ancestry(&self) -> &Ancestry;
    /// Get a mutable reference to property's ancestry
    fn get_ancestry_mut(&mut self) -> &mut Ancestry;
}

/// This must be implemented for all Properties
#[enum_dispatch]
pub trait PropertyTrait: PropertyDataTrait + Debug + Hash + Clone + PartialEq + Eq {
    /// Write property to an asset
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error>;
}

/// Property
#[allow(clippy::large_enum_variant)]
#[enum_dispatch(PropertyTrait, PropertyDataTrait)]
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
#[container_nobounds]
pub enum Property {
    /// Bool property
    BoolProperty,
    /// UInt16 property
    UInt16Property,
    /// UInt32 property
    UInt32Property,
    /// UInt64 property
    UInt64Property,
    /// Float property
    FloatProperty,
    /// Int16 property
    Int16Property,
    /// Int64 property
    Int64Property,
    /// Int8 property
    Int8Property,
    /// Int32 property
    IntProperty,
    /// Byte property
    ByteProperty,
    /// Double property
    DoubleProperty,
    /// Name property
    NameProperty,
    /// String property
    StrProperty,
    /// Text property
    TextProperty,
    /// Object property
    ObjectProperty,
    /// Asset object property
    AssetObjectProperty,
    /// Soft object property
    SoftObjectProperty,
    /// Int point property
    IntPointProperty,
    /// Vector property
    VectorProperty,
    /// Vector4 property
    Vector4Property,
    /// Vector2D property
    Vector2DProperty,
    /// Box property
    BoxProperty,
    /// Box2D property
    Box2DProperty,
    /// Quaternion property
    QuatProperty,
    /// Rotator property
    RotatorProperty,
    /// Linear color property
    LinearColorProperty,
    /// Color property
    ColorProperty,
    /// Timespan property
    TimeSpanProperty,
    /// Datetime property
    DateTimeProperty,
    /// Guid property
    GuidProperty,
    /// Set property
    SetProperty,
    /// Array property
    ArrayProperty,
    /// Map property
    MapProperty,
    /// Per-platform bool property
    PerPlatformBoolProperty,
    /// Per-platform int property
    PerPlatformIntProperty,
    /// Per-platform float property
    PerPlatformFloatProperty,
    /// Material attributes input property
    MaterialAttributesInputProperty,
    /// Expression input property
    ExpressionInputProperty,
    /// Color material input property
    ColorMaterialInputProperty,
    /// Scalar material input property
    ScalarMaterialInputProperty,
    /// Shading model material input property
    ShadingModelMaterialInputProperty,
    /// Vector material input property
    VectorMaterialInputProperty,
    /// Vector2 material input property
    Vector2MaterialInputProperty,
    /// Weighted random sampler property
    WeightedRandomSamplerProperty,
    /// Skeletal mesh sampling lod built data property
    SkeletalMeshSamplingLODBuiltDataProperty,
    /// Skeletal mesh area weighted triangle sampler
    SkeletalMeshAreaWeightedTriangleSampler,
    /// Soft asset path property
    SoftAssetPathProperty,
    /// Soft object path property
    SoftObjectPathProperty,
    /// Soft class path property
    SoftClassPathProperty,
    /// String asset reference property
    StringAssetReferenceProperty,
    /// Delegate property
    DelegateProperty,
    /// Multicast delegate property
    MulticastDelegateProperty,
    /// Multicast sparse delegate property
    MulticastSparseDelegateProperty,
    /// Multicast inline delegate property
    MulticastInlineDelegateProperty,
    /// Rich curve key property
    RichCurveKeyProperty,
    /// View target blend params property
    ViewTargetBlendParamsProperty,
    /// Gameplay tag container property
    GameplayTagContainerProperty,
    /// Smart name property
    SmartNameProperty,
    /// Struct property
    StructProperty,
    /// Enum property
    EnumProperty,
    /// Cloth lod data property
    ClothLodDataProperty,
    /// Font character property
    FontCharacterProperty,
    /// Unique net identifier property
    UniqueNetIdProperty,
    /// Niagara variable property
    NiagaraVariableProperty,
    /// Niagara variable with offset property
    NiagaraVariableWithOffsetProperty,
    /// Font data property
    FontDataProperty,
    /// Float range property
    FloatRangeProperty,
    /// Raw struct property
    RawStructProperty,
    /// Movie scene eval template pointer property
    MovieSceneEvalTemplatePtrProperty,
    /// Movie scene track implementation pointer property
    MovieSceneTrackImplementationPtrProperty,
    /// Movie scene evaluation field entity tree property
    MovieSceneEvaluationFieldEntityTreeProperty,
    /// Movie scene sub sequence tree property
    MovieSceneSubSequenceTreeProperty,
    /// Movie scene sequence instance data ptr property
    MovieSceneSequenceInstanceDataPtrProperty,
    /// Section evaluation data tree property
    SectionEvaluationDataTreeProperty,
    /// Movie scene track field data property
    MovieSceneTrackFieldDataProperty,
    /// Movie scene event parameters property
    MovieSceneEventParametersProperty,
    /// Movie scene float channel property
    MovieSceneFloatChannelProperty,
    /// Movie scene float value property
    MovieSceneFloatValueProperty,
    /// Movie scene frame range property
    MovieSceneFrameRangeProperty,
    /// Movie scene segment property
    MovieSceneSegmentProperty,
    /// Movie scene segment identifier property
    MovieSceneSegmentIdentifierProperty,
    /// Movie scene track identifier property
    MovieSceneTrackIdentifierProperty,
    /// Movie scene sequence id property
    MovieSceneSequenceIdProperty,
    /// Movie scene evaluation key property
    MovieSceneEvaluationKeyProperty,

    /// Empty unversioned property
    EmptyProperty,
    /// Unknown property
    UnknownProperty,
}

impl Property {
    /// Tries to read a property from an ArchiveReader
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        ancestry: Ancestry,
        unversioned_header: Option<&mut UnversionedHeader>,
        include_header: bool,
    ) -> Result<Option<Self>, Error> {
        let name: FName;
        let property_type: FName;
        let length: i32;
        let duplication_index: i32;
        let mut is_zero = false;

        if asset.has_unversioned_properties() {
            let header = unversioned_header.ok_or_else(PropertyError::no_unversioned_header)?;
            let mappings = asset
                .get_mappings()
                .ok_or_else(PropertyError::no_mappings)?;
            let parent_name = ancestry.get_parent().ok_or_else(PropertyError::no_parent)?;

            loop {
                let current_fragment = header.fragments[header.current_fragment_index];
                if header.unversioned_property_index > current_fragment.get_last_num() as usize {
                    break;
                }

                header.current_fragment_index += 1;
                header.unversioned_property_index =
                    header.fragments[header.current_fragment_index].first_num as usize;
            }

            let mut practicing_unversioned_property_index = header.unversioned_property_index;
            let mut schema = mappings
                .schemas
                .get_by_key(&parent_name.get_content())
                .ok_or_else(|| {
                    PropertyError::no_schema(
                        parent_name.get_content(),
                        practicing_unversioned_property_index,
                    )
                })?;

            while practicing_unversioned_property_index >= schema.prop_count as usize {
                practicing_unversioned_property_index -= schema.prop_count as usize;

                let new_schema =
                    mappings
                        .schemas
                        .get_by_key(&schema.super_type)
                        .ok_or_else(|| {
                            PropertyError::no_schema(
                                parent_name.get_content(),
                                practicing_unversioned_property_index,
                            )
                        })?;

                schema = new_schema;
            }

            let property = schema
                .properties
                .get_by_index(practicing_unversioned_property_index)
                .unwrap();
            header.unversioned_property_index += 1;

            name = FName::new_dummy(property.name.clone(), 0);
            property_type =
                FName::new_dummy(property.property_data.get_property_type().to_string(), 0);
            length = 1;
            duplication_index = property.array_index as i32;

            let current_fragment = header.fragments[header.current_fragment_index];
            if current_fragment.has_zeros {
                is_zero = match header.zero_mask_index < header.zero_mask.len() {
                    true => header.zero_mask[header.zero_mask_index],
                    false => false,
                };

                header.zero_mask_index += 1;
            }
        } else {
            name = asset.read_fname()?;
            if &name.get_content() == "None" {
                return Ok(None);
            }

            property_type = asset.read_fname()?;
            length = asset.read_i32::<LE>()?;
            duplication_index = asset.read_i32::<LE>()?;
        }

        Property::from_type(
            asset,
            &property_type,
            name,
            ancestry,
            include_header,
            length as i64,
            0,
            duplication_index,
            is_zero,
        )
        .map(Some)
    }

    /// Tries to read a property from an ArchiveReader while specified a type and length
    #[allow(clippy::too_many_arguments)]
    pub fn from_type<Reader: ArchiveReader>(
        asset: &mut Reader,
        type_name: &FName,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        length: i64,
        fallback_length: i64,
        duplication_index: i32,
        is_zero: bool,
    ) -> Result<Self, Error> {
        if is_zero {
            return Ok(EmptyProperty::new(type_name.clone(), name, ancestry).into());
        }

        let res = match type_name.get_content().as_str() {
            "BoolProperty" => BoolProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "UInt16Property" => UInt16Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "UInt32Property" => UInt32Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "UInt64Property" => UInt64Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "FloatProperty" => FloatProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "Int16Property" => Int16Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "Int64Property" => Int64Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "Int8Property" => Int8Property::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "IntProperty" => IntProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "ByteProperty" => ByteProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                fallback_length,
                duplication_index,
            )?
            .into(),
            "DoubleProperty" => DoubleProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "NameProperty" => {
                NameProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "StrProperty" => {
                StrProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "TextProperty" => {
                TextProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }

            "ObjectProperty" => {
                ObjectProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "AssetObjectProperty" => {
                AssetObjectProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "SoftObjectProperty" => {
                SoftObjectProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }

            "IntPoint" => {
                IntPointProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Vector" => {
                VectorProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Vector4" => {
                Vector4Property::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Vector2D" => {
                Vector2DProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Box" => {
                BoxProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "Box2D" => {
                Box2DProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "Quat" => {
                QuatProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "Rotator" => {
                RotatorProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "LinearColor" => {
                LinearColorProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Color" => {
                ColorProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }
            "Timespan" => {
                TimeSpanProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "DateTime" => {
                DateTimeProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "Guid" => {
                GuidProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }

            "SetProperty" => SetProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "ArrayProperty" => ArrayProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
                true,
            )?
            .into(),
            "MapProperty" => {
                MapProperty::new(asset, name, ancestry, include_header, duplication_index)?.into()
            }

            "PerPlatformBool" => PerPlatformBoolProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "PerPlatformInt" => PerPlatformIntProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "PerPlatformFloat" => PerPlatformFloatProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "MaterialAttributesInput" => MaterialAttributesInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "ExpressionInput" => ExpressionInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "ColorMaterialInput" => ColorMaterialInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "ScalarMaterialInput" => ScalarMaterialInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "ShadingModelMaterialInput" => ShadingModelMaterialInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "VectorMaterialInput" => VectorMaterialInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "Vector2MaterialInput" => Vector2MaterialInputProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),

            "WeightedRandomSampler" => WeightedRandomSamplerProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SkeletalMeshAreaWeightedTriangleSampler" => {
                SkeletalMeshAreaWeightedTriangleSampler::new(
                    asset,
                    name,
                    ancestry,
                    include_header,
                    length,
                    duplication_index,
                )?
                .into()
            }
            "SkeletalMeshSamplingLODBuiltData" => SkeletalMeshSamplingLODBuiltDataProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SoftAssetPath" => SoftAssetPathProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SoftObjectPath" => SoftObjectPathProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SoftClassPath" => SoftClassPathProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "StringAssetReference" => StringAssetReferenceProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "DelegateProperty" => DelegateProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "MulticastDelegateProperty" => MulticastDelegateProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "MulticastSparseDelegateProperty" => MulticastSparseDelegateProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "MulticastInlineDelegateProperty" => MulticastInlineDelegateProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "RichCurveKey" => RichCurveKeyProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "ViewTargetBlendParams" => ViewTargetBlendParamsProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "GameplayTagContainer" => GameplayTagContainerProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "SmartName" => SmartNameProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "StructProperty" => StructProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "EnumProperty" => EnumProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "ClothLODData" => ClothLodDataProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),

            "FontCharacter" => FontCharacterProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "UniqueNetIdRepl" => UniqueNetIdProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "NiagaraVariable" => NiagaraVariableProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "NiagaraVariableWithOffset" => NiagaraVariableWithOffsetProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "FontData" => FontDataProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
            )?
            .into(),
            "FloatRange" => {
                FloatRangeProperty::new(asset, name, ancestry, include_header, duplication_index)?
                    .into()
            }
            "RawStructProperty" => RawStructProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
                length,
            )?
            .into(),

            "MovieSceneEvalTemplatePtr" => MovieSceneEvalTemplatePtrProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneTrackImplementationPtr" => MovieSceneTrackImplementationPtrProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneEvaluationFieldEntityTree" => {
                MovieSceneEvaluationFieldEntityTreeProperty::new(
                    asset,
                    name,
                    ancestry,
                    include_header,
                    duplication_index,
                )?
                .into()
            }
            "MovieSceneSubSequenceTree" => MovieSceneSubSequenceTreeProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneSequenceInstanceDataPtr" => MovieSceneSequenceInstanceDataPtrProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "SectionEvaluationDataTree" => SectionEvaluationDataTreeProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneTrackFieldData" => MovieSceneTrackFieldDataProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneEventParameters" => MovieSceneEventParametersProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneFloatChannel" => MovieSceneFloatChannelProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneFloatValue" => MovieSceneFloatValueProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneFrameRange" => MovieSceneFrameRangeProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneSegment" => MovieSceneSegmentProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneSegmentIdentifier" => MovieSceneSegmentIdentifierProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneTrackIdentifier" => MovieSceneTrackIdentifierProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneSequenceId" => MovieSceneSequenceIdProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),
            "MovieSceneEvaluationKey" => MovieSceneEvaluationKeyProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                duplication_index,
            )?
            .into(),

            _ => UnknownProperty::new(
                asset,
                name,
                ancestry,
                include_header,
                length,
                duplication_index,
                type_name.clone(),
            )?
            .into(),
        };

        Ok(res)
    }

    /// Writes a property to an ArchiveWriter
    pub fn write<Writer: ArchiveWriter>(
        property: &Property,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        asset.write_fname(&property.get_name())?;

        let property_serialized_name = property.to_serialized_name();
        asset.write_fname(
            &asset
                .get_name_map()
                .get_mut()
                .add_fname(&property_serialized_name),
        )?;

        let begin = asset.position();
        asset.write_i32::<LE>(0)?; // initial length
        asset.write_i32::<LE>(property.get_duplication_index())?;
        let len = property.write(asset, include_header)?;
        let end = asset.position();

        asset.seek(SeekFrom::Start(begin))?;
        asset.write_i32::<LE>(len as i32)?;
        asset.seek(SeekFrom::Start(end))?;
        Ok(begin as usize)
    }

    /// Check if a property type has custom serialization
    pub fn has_custom_serialization(name: &String) -> bool {
        CUSTOM_SERIALIZATION.contains(name)
    }
}

/// Implements `ToSerializedName` trait for properties
macro_rules! property_inner_serialized_name {
    ($($inner:ident : $name:expr),*) => {
        impl ToSerializedName for Property {
            fn to_serialized_name(&self) -> String {
                match self {
                    $(
                        Self::$inner(_) => String::from($name),
                    )*
                    Self::UnknownProperty(unk) => unk
                        .serialized_type.get_content(),
                    Self::EmptyProperty(empty) => empty.type_name.get_content()
                }
            }
        }
    };
}

property_inner_serialized_name! {
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
    ClothLodDataProperty: "ClothLODData",
    FloatProperty: "FloatProperty",
    Int16Property: "Int16Property",
    Int64Property: "Int64Property",
    Int8Property: "Int8Property",
    IntProperty: "IntProperty",
    MapProperty: "MapProperty",
    MulticastDelegateProperty: "MulticastDelegateProperty",
    MulticastSparseDelegateProperty: "MulticastSparseDelegateProperty",
    MulticastInlineDelegateProperty: "MulticastInlineDelegateProperty",
    DelegateProperty: "DelegateProperty",
    NameProperty: "NameProperty",
    ObjectProperty: "ObjectProperty",
    AssetObjectProperty: "AssetObjectProperty",
    SoftObjectProperty: "SoftObjectProperty",
    StrProperty: "StrProperty",
    TextProperty: "TextProperty",
    UInt16Property: "UInt16Property",
    UInt32Property: "UInt32Property",
    UInt64Property: "UInt64Property",

    FontCharacterProperty: "FontCharacter",
    UniqueNetIdProperty: "UniqueNetIdRepl",
    NiagaraVariableProperty: "NiagaraVariable",
    NiagaraVariableWithOffsetProperty: "NiagaraVariableWithOffset",
    FontDataProperty: "FontData",
    FloatRangeProperty: "FloatRange",
    RawStructProperty: "RawStructProperty",

    MovieSceneEvalTemplatePtrProperty: "MovieSceneEvalTemplatePtr",
    MovieSceneTrackImplementationPtrProperty: "MovieSceneTrackImplementationPtr",
    MovieSceneEvaluationFieldEntityTreeProperty: "MovieSceneEvaluationFieldEntityTree",
    MovieSceneSubSequenceTreeProperty: "MovieSceneSubSequenceTree",
    MovieSceneSequenceInstanceDataPtrProperty: "MovieSceneSequenceInstanceDataPtr",
    SectionEvaluationDataTreeProperty: "SectionEvaluationDataTree",
    MovieSceneTrackFieldDataProperty: "MovieSceneTrackFieldData",
    MovieSceneEventParametersProperty: "MovieSceneEventParameters",
    MovieSceneFloatChannelProperty: "MovieSceneFloatChannel",
    MovieSceneFloatValueProperty: "MovieSceneFloatValue",
    MovieSceneFrameRangeProperty: "MovieSceneFrameRange",
    MovieSceneSegmentProperty: "MovieSceneSegment",
    MovieSceneSegmentIdentifierProperty: "MovieSceneSegmentIdentifier",
    MovieSceneTrackIdentifierProperty: "MovieSceneTrackIdentifier",
    MovieSceneSequenceIdProperty: "MovieSceneSequenceId",
    MovieSceneEvaluationKeyProperty: "MovieSceneEvaluationKey"
}
