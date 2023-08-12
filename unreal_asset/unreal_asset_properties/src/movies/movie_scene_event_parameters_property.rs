//! Movie scene event parameters property

use crate::property_prelude::*;

/// Movie scene event parameters
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEventParameters {
    /// Struct type
    pub struct_type: SoftObjectPath,
    /// Struct raw data
    pub struct_bytes: Vec<u8>,
}

impl MovieSceneEventParameters {
    /// Read `MovieSceneEventParameters` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let struct_type = SoftObjectPath::new(asset)?;

        let struct_bytes_length = asset.read_i32::<LE>()?;
        let mut struct_bytes = vec![0u8; struct_bytes_length as usize];
        asset.read_exact(&mut struct_bytes)?;

        Ok(MovieSceneEventParameters {
            struct_type,
            struct_bytes,
        })
    }

    /// Write `MovieSceneEventParameters` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_type.write(asset)?;

        asset.write_i32::<LE>(self.struct_bytes.len() as i32)?;
        asset.write_all(&self.struct_bytes)?;

        Ok(())
    }
}

/// Movie scene event parameters property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEventParametersProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: MovieSceneEventParameters,
}
impl_property_data_trait!(MovieSceneEventParametersProperty);

impl MovieSceneEventParametersProperty {
    /// Read `MovieSceneEventParametersProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = MovieSceneEventParameters::new(asset)?;

        Ok(MovieSceneEventParametersProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEventParametersProperty {
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
