use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{core_uobject::ERangeBoundTypes, PropertyTrait},
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Int32RangeBound {
    pub ty: ERangeBoundTypes,
    pub value: i32,
}

impl Int32RangeBound {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let ty: ERangeBoundTypes = ERangeBoundTypes::try_from(asset.read_i8()?)?;
        let value = asset.read_i32::<LittleEndian>()?;

        Ok(Int32RangeBound { ty, value })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i8(self.ty as i8)?;
        asset.write_i32::<LittleEndian>(self.value)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFrameRangeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,

    pub lower_bound: Int32RangeBound,
    pub upper_bound: Int32RangeBound,
}
impl_property_data_trait!(MovieSceneFrameRangeProperty);

impl MovieSceneFrameRangeProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let lower_bound = Int32RangeBound::new(asset)?;
        let upper_bound = Int32RangeBound::new(asset)?;

        Ok(MovieSceneFrameRangeProperty {
            name,
            property_guid,
            duplication_index,
            lower_bound,
            upper_bound,
        })
    }
}

impl PropertyTrait for MovieSceneFrameRangeProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.lower_bound.write(asset)?;
        self.upper_bound.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
