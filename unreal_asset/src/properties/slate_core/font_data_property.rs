use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    custom_version::FEditorObjectVersion,
    error::{Error, PropertyError},
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid, PackageIndex},
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EFontHinting {
    /// Use the default hinting specified in the font.
    Default,
    /// Force the use of an automatic hinting algorithm.
    Auto,
    /// Force the use of an automatic light hinting algorithm, optimized for non-monochrome displays.
    AutoLight,
    /// Force the use of an automatic hinting algorithm optimized for monochrome displays.
    Monochrome,
    /// Do not use hinting.
    None,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EFontLoadingPolicy {
    /// Lazy load the entire font into memory. This will consume more memory than Streaming, however there will be zero file-IO when rendering glyphs within the font, although the initial load may cause a hitch.
    LazyLoad,
    /// Stream the font from disk. This will consume less memory than LazyLoad or Inline, however there will be file-IO when rendering glyphs, which may cause hitches under certain circumstances or on certain platforms.
    Stream,
    /// Embed the font data within the asset. This will consume more memory than Streaming, however it is guaranteed to be hitch free (only valid for font data within a Font Face asset).
    Inline,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontData {
    /// UObject
    local_font_face_asset: PackageIndex,
    font_filename: Option<String>,
    hinting: Option<EFontHinting>,
    loading_policy: Option<EFontLoadingPolicy>,
    sub_face_index: Option<i32>,
    is_cooked: bool,
}

impl FontData {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Option<Self>, Error> {
        if asset.get_custom_version::<FEditorObjectVersion>().version
            < FEditorObjectVersion::AddedFontFaceAssets as i32
        {
            return Ok(None);
        }

        let is_cooked = asset.read_i32::<LittleEndian>()? != 0;

        let mut local_font_face_asset = PackageIndex::new(0);
        let mut font_filename = None;
        let mut hinting: Option<EFontHinting> = None;
        let mut loading_policy: Option<EFontLoadingPolicy> = None;
        let mut sub_face_index = None;

        if is_cooked {
            local_font_face_asset = PackageIndex::new(asset.read_i32::<LittleEndian>()?);

            if local_font_face_asset.index == 0 {
                font_filename = asset.read_string()?;
                hinting = Some(EFontHinting::try_from(asset.read_u8()?)?);
                loading_policy = Some(EFontLoadingPolicy::try_from(asset.read_u8()?)?);
            }

            sub_face_index = Some(asset.read_i32::<LittleEndian>()?);
        }

        Ok(Some(FontData {
            local_font_face_asset,
            font_filename,
            hinting,
            loading_policy,
            sub_face_index,
            is_cooked,
        }))
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        if asset.get_custom_version::<FEditorObjectVersion>().version
            < FEditorObjectVersion::AddedFontFaceAssets as i32
        {
            return Ok(());
        }

        asset.write_i32::<LittleEndian>(match self.is_cooked {
            true => 1,
            false => 0,
        })?;

        if self.is_cooked {
            asset.write_i32::<LittleEndian>(self.local_font_face_asset.index)?;

            if self.local_font_face_asset.index == 0 {
                asset.write_string(&self.font_filename)?;
                asset.write_u8(self.hinting.ok_or_else(|| {
                    PropertyError::property_field_none("hinting", "Some(EFontHinting)")
                })? as u8)?;
                asset.write_u8(self.loading_policy.ok_or_else(|| {
                    PropertyError::property_field_none("loading_policy", "Some(ELoadingPolicy)")
                })? as u8)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontDataProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<FontData>,
}
impl_property_data_trait!(FontDataProperty);

impl FontDataProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = FontData::new(asset)?;

        Ok(FontDataProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for FontDataProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        if let Some(value) = self.value.as_ref() {
            value.write(asset)?;
        }

        Ok((asset.position() - begin) as usize)
    }
}
