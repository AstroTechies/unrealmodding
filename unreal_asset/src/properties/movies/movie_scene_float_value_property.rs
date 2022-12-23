use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{
        rich_curve_key_property::{RichCurveInterpMode, RichCurveTangentMode},
        PropertyTrait,
    },
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

use super::MovieSceneTangentData;

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
pub struct MovieSceneFloatValueProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneFloatValue,
}
impl_property_data_trait!(MovieSceneFloatValueProperty);

impl MovieSceneFloatValueProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        // todo: clangwin64 is always false?
        let value = MovieSceneFloatValue::new(asset, false)?;

        Ok(MovieSceneFloatValueProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneFloatValueProperty {
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
