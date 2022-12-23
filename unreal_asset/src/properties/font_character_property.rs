use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

use super::PropertyTrait;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FontCharacter {
    pub start_u: i32,
    pub start_v: i32,
    pub size_u: i32,
    pub size_v: i32,
    pub texture_index: u8,
    pub vertical_offset: i32,
}

impl FontCharacter {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(FontCharacter {
            start_u: asset.read_i32::<LittleEndian>()?,
            start_v: asset.read_i32::<LittleEndian>()?,
            size_u: asset.read_i32::<LittleEndian>()?,
            size_v: asset.read_i32::<LittleEndian>()?,
            texture_index: asset.read_u8()?,
            vertical_offset: asset.read_i32::<LittleEndian>()?,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.start_u)?;
        asset.write_i32::<LittleEndian>(self.start_v)?;
        asset.write_i32::<LittleEndian>(self.size_u)?;
        asset.write_i32::<LittleEndian>(self.size_v)?;
        asset.write_u8(self.texture_index)?;
        asset.write_i32::<LittleEndian>(self.vertical_offset)?;
        Ok(())
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FontCharacterProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: FontCharacter,
}
impl_property_data_trait!(FontCharacterProperty);

impl FontCharacterProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = FontCharacter::new(asset)?;

        Ok(FontCharacterProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for FontCharacterProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, crate::error::Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.value.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
