use std::default;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{rich_curve_key_property::RichCurveExtrapolation, PropertyTrait},
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, FrameNumber, FrameRate, Guid},
};

use super::MovieSceneFloatValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatChannel {
    pub pre_infinity_extrap: RichCurveExtrapolation,
    pub post_infinity_extrap: RichCurveExtrapolation,

    pub times_struct_length: i32,
    pub times: Vec<FrameNumber>,

    pub values_struct_length: i32,
    pub values: Vec<MovieSceneFloatValue>,

    pub default_value: OrderedFloat<f32>,
    pub has_default_value: bool,
    pub tick_resolution: FrameRate,
}

impl MovieSceneFloatChannel {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let pre_infinity_extrap: RichCurveExtrapolation =
            RichCurveExtrapolation::try_from(asset.read_u8()?)?;
        let post_infinity_extrap: RichCurveExtrapolation =
            RichCurveExtrapolation::try_from(asset.read_u8()?)?;

        let times_struct_length = asset.read_i32::<LittleEndian>()?;
        let times_length = asset.read_i32::<LittleEndian>()?;

        let mut times = Vec::with_capacity(times_length as usize);
        for _ in 0..times_length {
            times.push(FrameNumber::new(asset.read_i32::<LittleEndian>()?));
        }

        let values_struct_length = asset.read_i32::<LittleEndian>()?;
        let values_length = asset.read_i32::<LittleEndian>()?;

        let mut values = Vec::with_capacity(values_length as usize);
        for _ in 0..values_length {
            //todo: clangwin64 is always false?
            values.push(MovieSceneFloatValue::new(asset, false)?);
        }

        let default_value = asset.read_f32::<LittleEndian>()?;
        let has_default_value = asset.read_i32::<LittleEndian>()? == 1;

        let tick_resolution = FrameRate::new(
            asset.read_i32::<LittleEndian>()?,
            asset.read_i32::<LittleEndian>()?,
        );

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

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_u8(self.pre_infinity_extrap as u8)?;
        asset.write_u8(self.post_infinity_extrap as u8)?;

        asset.write_i32::<LittleEndian>(self.times_struct_length)?;
        asset.write_i32::<LittleEndian>(self.times.len() as i32)?;

        for time in &self.times {
            asset.write_i32::<LittleEndian>(time.value)?;
        }

        asset.write_i32::<LittleEndian>(self.values_struct_length)?;
        asset.write_i32::<LittleEndian>(self.values.len() as i32)?;

        for value in &self.values {
            value.write(asset)?;
        }

        asset.write_f32::<LittleEndian>(self.default_value.0)?;
        asset.write_i32::<LittleEndian>(match self.has_default_value {
            true => 1,
            false => 0,
        })?;

        asset.write_i32::<LittleEndian>(self.tick_resolution.numerator)?;
        asset.write_i32::<LittleEndian>(self.tick_resolution.denominator)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatChannelProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneFloatChannel,
}
impl_property_data_trait!(MovieSceneFloatChannelProperty);

impl MovieSceneFloatChannelProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneFloatChannel::new(asset)?;

        Ok(MovieSceneFloatChannelProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneFloatChannelProperty {
    fn write<Writer: AssetWriter>(
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
