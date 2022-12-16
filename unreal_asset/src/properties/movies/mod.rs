use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::{
    error::Error,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FrameNumber, FrameRate},
};

use super::rich_curve_key_property::{
    RichCurveExtrapolation, RichCurveInterpMode, RichCurveTangentMode, RichCurveTangentWeightMode,
};

pub mod enums;
pub mod movie_scene_eval_template_ptr_property;
pub mod movie_scene_evaluation;
pub mod movie_scene_evaluation_field_entity_tree_property;
pub mod movie_scene_sequence_instance_data_ptr_property;
pub mod movie_scene_sub_sequence_tree_property;
pub mod movie_scene_track_field_data_property;
pub mod movie_scene_track_implementation_ptr_property;
pub mod section_evaluation_data_tree_property;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTangentData {
    pub arrive_tangent: OrderedFloat<f32>,
    pub leave_tangent: OrderedFloat<f32>,
    pub arrive_tangent_weight: OrderedFloat<f32>,
    pub leave_tangent_weight: OrderedFloat<f32>,
    pub tangent_weight_mode: RichCurveTangentWeightMode,
    pub padding: Vec<u8>,

    clang_win64: bool,
}

impl MovieSceneTangentData {
    pub fn new<Reader: AssetReader>(asset: &mut Reader, clang_win64: bool) -> Result<Self, Error> {
        let arrive_tangent = asset.read_f32::<LittleEndian>()?;
        let leave_tangent = asset.read_f32::<LittleEndian>()?;
        let arrive_tangent_weight = asset.read_f32::<LittleEndian>()?;
        let leave_tangent_weight = asset.read_f32::<LittleEndian>()?;
        let tangent_weight_mode: RichCurveTangentWeightMode =
            RichCurveTangentWeightMode::try_from(asset.read_i8()?)?;
        let padding = match clang_win64 {
            true => vec![0u8; 3],
            false => vec![0u8; 0],
        };
        if clang_win64 {
            asset.read_exact(&mut padding)?;
        }

        Ok(MovieSceneTangentData {
            arrive_tangent: OrderedFloat(arrive_tangent),
            leave_tangent: OrderedFloat(leave_tangent),
            arrive_tangent_weight: OrderedFloat(arrive_tangent_weight),
            leave_tangent_weight: OrderedFloat(leave_tangent_weight),
            tangent_weight_mode,
            padding,
            clang_win64,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_f32::<LittleEndian>(self.arrive_tangent.0)?;
        asset.write_f32::<LittleEndian>(self.leave_tangent.0)?;
        asset.write_f32::<LittleEndian>(self.arrive_tangent_weight.0)?;
        asset.write_f32::<LittleEndian>(self.leave_tangent_weight.0)?;
        asset.write_i8(self.tangent_weight_mode as i8)?;

        if self.clang_win64 {
            asset.write_all(&self.padding)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatValue {
    pub value: OrderedFloat<f32>,
    pub tangent: MovieSceneTangentData,
    pub interp_mode: RichCurveInterpMode,
    pub tangent_mode: RichCurveTangentMode,
}

impl MovieSceneFloatValue {
    pub fn new<Reader: AssetReader>(asset: &mut Reader, clang_win64: bool) -> Result<Self, Error> {
        let value = asset.read_f32::<LittleEndian>()?;
        let tangent = MovieSceneTangentData::new(asset, clang_win64)?;
        let interp_mode: RichCurveInterpMode = RichCurveInterpMode::try_from(asset.read_i8()?)?;
        let tangent_mode: RichCurveTangentMode = RichCurveTangentMode::try_from(asset.read_i8()?)?;

        Ok(MovieSceneFloatValue {
            value: OrderedFloat(value),
            tangent,
            interp_mode,
            tangent_mode,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_f32::<LittleEndian>(self.value.0)?;
        self.tangent.write(asset)?;
        asset.write_i8(self.interp_mode as i8)?;
        asset.write_i8(self.tangent_mode as i8)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatChannel {
    pub pre_infinity_extrap: RichCurveExtrapolation,
    pub post_infinity_extrap: RichCurveExtrapolation,
    pub times: Vec<FrameNumber>,
    pub values: Vec<MovieSceneFloatValue>,
    pub default_value: OrderedFloat<f32>,
    pub has_default_value: bool,
    pub tick_resolution: FrameRate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackIdentifier {
    pub value: u32,
}

impl MovieSceneTrackIdentifier {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let value = asset.read_u32::<LittleEndian>()?;

        Ok(MovieSceneTrackIdentifier { value })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_u32::<LittleEndian>(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSequenceId {
    pub value: u32,
}

impl MovieSceneSequenceId {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let value = asset.read_u32::<LittleEndian>()?;

        Ok(MovieSceneSequenceId { value })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_u32::<LittleEndian>(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationKey {
    pub sequence_id: MovieSceneSequenceId,
    pub track_identifier: MovieSceneTrackIdentifier,
    pub section_index: u32,
}

impl MovieSceneEvaluationKey {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let sequence_id = MovieSceneSequenceId::new(asset)?;
        let track_identifier = MovieSceneTrackIdentifier::new(asset)?;
        let section_index = asset.read_u32::<LittleEndian>()?;

        Ok(MovieSceneEvaluationKey {
            sequence_id,
            track_identifier,
            section_index,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.sequence_id.write(asset)?;
        self.track_identifier.write(asset)?;
        asset.write_u32::<LittleEndian>(self.section_index)?;

        Ok(())
    }
}
