use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

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
pub struct MovieSceneSequenceIdProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneSequenceId,
}
impl_property_data_trait!(MovieSceneSequenceIdProperty);

impl MovieSceneSequenceIdProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSequenceId::new(asset)?;

        Ok(MovieSceneSequenceIdProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSequenceIdProperty {
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
