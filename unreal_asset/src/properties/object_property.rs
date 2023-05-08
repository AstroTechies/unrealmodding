//! Object properties

use std::mem::size_of;

use byteorder::LE;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, Guid, PackageIndex};
use crate::unversioned::ancestry::Ancestry;

/// Object property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ObjectProperty {
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
    pub value: PackageIndex,
}
impl_property_data_trait!(ObjectProperty);

/// Asset object property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AssetObjectProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: Option<String>,
}
impl_property_data_trait!(AssetObjectProperty);

/// Soft object path
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SoftObjectPath {
    /// Asset path name
    pub asset_path_name: FName,
    /// Sub path string
    pub sub_path_string: Option<String>,
}

impl SoftObjectPath {
    /// Read a `SoftObjectPath` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let asset_path_name = asset.read_fname()?;
        let sub_path_string = asset.read_fstring()?;

        Ok(SoftObjectPath {
            asset_path_name,
            sub_path_string,
        })
    }

    /// Write a `SoftObjectPath` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_fname(&self.asset_path_name)?;
        asset.write_fstring(self.sub_path_string.as_deref())?;

        Ok(())
    }
}

/// Soft object property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SoftObjectProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Soft object path value
    pub value: SoftObjectPath,
}
impl_property_data_trait!(SoftObjectProperty);

impl ObjectProperty {
    /// Read an `ObjectProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_i32::<LE>()?;
        Ok(ObjectProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: PackageIndex::new(value),
        })
    }
}

impl PropertyTrait for ObjectProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LE>(self.value.index)?;
        Ok(size_of::<i32>())
    }
}

impl AssetObjectProperty {
    /// Read an `AssetObjectProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fstring()?;
        Ok(AssetObjectProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for AssetObjectProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_fstring(self.value.as_deref())
    }
}

impl SoftObjectProperty {
    /// Read a `SoftObjectProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = SoftObjectPath::new(asset)?;

        Ok(SoftObjectProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for SoftObjectProperty {
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
