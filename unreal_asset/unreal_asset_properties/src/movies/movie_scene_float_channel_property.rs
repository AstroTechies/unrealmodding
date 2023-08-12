//! Movie scene float channel property

use unreal_asset_base::types::movie::{FrameNumber, FrameRate};

use crate::property_prelude::*;
use crate::rich_curve_key_property::RichCurveExtrapolation;

use super::movie_scene_float_value_property::MovieSceneFloatValue;

/// Movie scene float channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatChannel {
    /// Pre infinity extrapolation
    pub pre_infinity_extrap: RichCurveExtrapolation,
    /// Post infinity extrapolation
    pub post_infinity_extrap: RichCurveExtrapolation,

    /// Frame times structure length
    pub times_struct_length: i32,
    /// Frame times
    pub times: Vec<FrameNumber>,

    /// Values structure length
    pub values_struct_length: i32,
    /// Values
    pub values: Vec<MovieSceneFloatValue>,

    /// Default value
    pub default_value: OrderedFloat<f32>,
    /// Has default value
    pub has_default_value: bool,
    /// Tick resolution
    pub tick_resolution: FrameRate,
}

impl MovieSceneFloatChannel {
    /// Read a `MovieSceneFloatChannel` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let pre_infinity_extrap: RichCurveExtrapolation =
            RichCurveExtrapolation::try_from(asset.read_u8()?)?;
        let post_infinity_extrap: RichCurveExtrapolation =
            RichCurveExtrapolation::try_from(asset.read_u8()?)?;

        let times_struct_length = asset.read_i32::<LE>()?;
        let times_length = asset.read_i32::<LE>()?;

        let mut times = Vec::with_capacity(times_length as usize);
        for _ in 0..times_length {
            times.push(FrameNumber::new(asset.read_i32::<LE>()?));
        }

        let values_struct_length = asset.read_i32::<LE>()?;
        let values_length = asset.read_i32::<LE>()?;

        let mut values = Vec::with_capacity(values_length as usize);
        for _ in 0..values_length {
            //todo: clangwin64 is always false?
            values.push(MovieSceneFloatValue::new(asset, false)?);
        }

        let default_value = asset.read_f32::<LE>()?;
        let has_default_value = asset.read_i32::<LE>()? == 1;

        let tick_resolution = FrameRate::new(asset.read_i32::<LE>()?, asset.read_i32::<LE>()?);

        Ok(MovieSceneFloatChannel {
            pre_infinity_extrap,
            post_infinity_extrap,
            times_struct_length,
            times,
            values_struct_length,
            values,
            default_value: OrderedFloat(default_value),
            has_default_value,
            tick_resolution,
        })
    }

    /// Write a `MovieSceneFloatChannel` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_u8(self.pre_infinity_extrap as u8)?;
        asset.write_u8(self.post_infinity_extrap as u8)?;

        asset.write_i32::<LE>(self.times_struct_length)?;
        asset.write_i32::<LE>(self.times.len() as i32)?;

        for time in &self.times {
            asset.write_i32::<LE>(time.value)?;
        }

        asset.write_i32::<LE>(self.values_struct_length)?;
        asset.write_i32::<LE>(self.values.len() as i32)?;

        for value in &self.values {
            value.write(asset)?;
        }

        asset.write_f32::<LE>(self.default_value.0)?;
        asset.write_i32::<LE>(match self.has_default_value {
            true => 1,
            false => 0,
        })?;

        asset.write_i32::<LE>(self.tick_resolution.numerator)?;
        asset.write_i32::<LE>(self.tick_resolution.denominator)?;

        Ok(())
    }
}

/// Movie scene float channel property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatChannelProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    #[container_ignore]
    pub value: MovieSceneFloatChannel,
}
impl_property_data_trait!(MovieSceneFloatChannelProperty);

impl MovieSceneFloatChannelProperty {
    /// Read a `MovieSceneFloatChannelProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneFloatChannel::new(asset)?;

        Ok(MovieSceneFloatChannelProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneFloatChannelProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.value.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
