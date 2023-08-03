//! Movie scene float value property

use byteorder::LE;
use ordered_float::OrderedFloat;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{
        rich_curve_key_property::{RichCurveInterpMode, RichCurveTangentMode},
        PropertyTrait,
    },
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::fname::FName,
    unversioned::ancestry::Ancestry,
};

use super::MovieSceneTangentData;

/// Movie scene float value
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatValue {
    /// Value
    pub value: OrderedFloat<f32>,
    /// Tangent
    pub tangent: MovieSceneTangentData,
    /// Interpolation mode
    pub interp_mode: RichCurveInterpMode,
    /// Tangent mode
    pub tangent_mode: RichCurveTangentMode,
}

impl MovieSceneFloatValue {
    /// Read a `MovieSceneFloatValue` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        clang_win64: bool,
    ) -> Result<Self, Error> {
        let value = asset.read_f32::<LE>()?;
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

    /// Write a `MovieSceneFloatValue` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_f32::<LE>(self.value.0)?;
        self.tangent.write(asset)?;
        asset.write_i8(self.interp_mode as i8)?;
        asset.write_i8(self.tangent_mode as i8)?;
        Ok(())
    }
}

/// Movie scene float value property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFloatValueProperty {
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
    pub value: MovieSceneFloatValue,
}
impl_property_data_trait!(MovieSceneFloatValueProperty);

impl MovieSceneFloatValueProperty {
    /// Read a `MovieSceneFloatValueProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        // todo: clangwin64 is always false?
        let value = MovieSceneFloatValue::new(asset, false)?;

        Ok(MovieSceneFloatValueProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneFloatValueProperty {
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
