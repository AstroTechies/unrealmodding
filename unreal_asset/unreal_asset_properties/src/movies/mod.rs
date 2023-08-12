//! Unreal movies

use crate::property_prelude::*;

use super::rich_curve_key_property::RichCurveTangentWeightMode;

pub mod enums;
pub mod movie_scene_eval_template_ptr_property;
pub mod movie_scene_evaluation;
pub mod movie_scene_evaluation_field_entity_tree_property;
pub mod movie_scene_evaluation_key_property;
pub mod movie_scene_event_parameters_property;
pub mod movie_scene_float_channel_property;
pub mod movie_scene_float_value_property;
pub mod movie_scene_frame_range_property;
pub mod movie_scene_segment_property;
pub mod movie_scene_sequence_id_property;
pub mod movie_scene_sequence_instance_data_ptr_property;
pub mod movie_scene_sub_sequence_tree_property;
pub mod movie_scene_track_field_data_property;
pub mod movie_scene_track_identifier_property;
pub mod movie_scene_track_implementation_ptr_property;
pub mod section_evaluation_data_tree_property;

/// Movie scene tangent data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTangentData {
    /// Arrive tangent
    pub arrive_tangent: OrderedFloat<f32>,
    /// Leave tangent
    pub leave_tangent: OrderedFloat<f32>,
    /// Arrive tangent weight
    pub arrive_tangent_weight: OrderedFloat<f32>,
    /// Leave tangent weight
    pub leave_tangent_weight: OrderedFloat<f32>,
    /// Tangent weight mode
    pub tangent_weight_mode: RichCurveTangentWeightMode,
    /// Padding
    pub padding: Vec<u8>,
    /// Is compiled with clang win64
    clang_win64: bool,
}

impl MovieSceneTangentData {
    /// Read `MovieSceneTangentData` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        clang_win64: bool,
    ) -> Result<Self, Error> {
        let arrive_tangent = asset.read_f32::<LE>()?;
        let leave_tangent = asset.read_f32::<LE>()?;
        let arrive_tangent_weight = asset.read_f32::<LE>()?;
        let leave_tangent_weight = asset.read_f32::<LE>()?;
        let tangent_weight_mode: RichCurveTangentWeightMode =
            RichCurveTangentWeightMode::try_from(asset.read_i8()?)?;
        let mut padding = match clang_win64 {
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

    /// Write `MovieSceneTangentData` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_f32::<LE>(self.arrive_tangent.0)?;
        asset.write_f32::<LE>(self.leave_tangent.0)?;
        asset.write_f32::<LE>(self.arrive_tangent_weight.0)?;
        asset.write_f32::<LE>(self.leave_tangent_weight.0)?;
        asset.write_i8(self.tangent_weight_mode as i8)?;

        if self.clang_win64 {
            asset.write_all(&self.padding)?;
        }
        Ok(())
    }
}
