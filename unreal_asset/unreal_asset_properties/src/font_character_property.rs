//! Font character property

use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Font character
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FontCharacter {
    /// Start U coordinate
    pub start_u: i32,
    /// Start V coordinate
    pub start_v: i32,
    /// U coordinate size
    pub size_u: i32,
    /// V coordinate size
    pub size_v: i32,
    /// Texture index
    pub texture_index: u8,
    /// Vertical offset
    pub vertical_offset: i32,
}

impl FontCharacter {
    /// Read a `FontCharacter` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        Ok(FontCharacter {
            start_u: asset.read_i32::<LE>()?,
            start_v: asset.read_i32::<LE>()?,
            size_u: asset.read_i32::<LE>()?,
            size_v: asset.read_i32::<LE>()?,
            texture_index: asset.read_u8()?,
            vertical_offset: asset.read_i32::<LE>()?,
        })
    }

    /// Write a `FontCharacter` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.start_u)?;
        asset.write_i32::<LE>(self.start_v)?;
        asset.write_i32::<LE>(self.size_u)?;
        asset.write_i32::<LE>(self.size_v)?;
        asset.write_u8(self.texture_index)?;
        asset.write_i32::<LE>(self.vertical_offset)?;
        Ok(())
    }
}

/// Font character property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct FontCharacterProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Font character
    #[container_ignore]
    pub value: FontCharacter,
}
impl_property_data_trait!(FontCharacterProperty);

impl FontCharacterProperty {
    /// Read a `FontCharacterProperty` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = FontCharacter::new(asset)?;

        Ok(FontCharacterProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for FontCharacterProperty {
    fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
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
