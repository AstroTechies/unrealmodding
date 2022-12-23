use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackIdentifierProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneTrackIdentifier,
}
impl_property_data_trait!(MovieSceneTrackIdentifierProperty);

impl MovieSceneTrackIdentifierProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneTrackIdentifier::new(asset)?;

        Ok(MovieSceneTrackIdentifierProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneTrackIdentifierProperty {
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
