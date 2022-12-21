use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{object_property::SoftObjectPath, PropertyTrait},
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEventParameters {
    pub struct_type: SoftObjectPath,
    pub struct_bytes: Vec<u8>,
}

impl MovieSceneEventParameters {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let struct_type = SoftObjectPath::new(asset)?;

        let struct_bytes_length = asset.read_i32::<LittleEndian>()?;
        let mut struct_bytes = vec![0u8; struct_bytes_length as usize];
        asset.read_exact(&mut struct_bytes)?;

        Ok(MovieSceneEventParameters {
            struct_type,
            struct_bytes,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_type.write(asset)?;

        asset.write_i32::<LittleEndian>(self.struct_bytes.len() as i32)?;
        asset.write_all(&self.struct_bytes)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEventParametersProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneEventParameters,
}
impl_property_data_trait!(MovieSceneEventParametersProperty);

impl MovieSceneEventParametersProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = MovieSceneEventParameters::new(asset)?;

        Ok(MovieSceneEventParametersProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEventParametersProperty {
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
