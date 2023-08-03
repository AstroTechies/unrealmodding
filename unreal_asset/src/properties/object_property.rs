//! Object properties

use std::mem::size_of;

use byteorder::LE;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersionUE5;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, PackageIndex};
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

/// Top level asset path
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct TopLevelAssetPath {
    /// Package name that contains the asset e.g. /Some/Path/Package
    /// Only present in 5.1 and higher
    pub package_name: Option<FName>,
    /// If 5.1 and higher contains the name of the asset within the package
    /// If less than 5.1 contians the full path to the asset
    pub asset_name: FName,
}

impl TopLevelAssetPath {
    /// Create a new `TopLevelAssetPath` instance
    pub fn new(package_name: Option<FName>, asset_name: FName) -> Self {
        TopLevelAssetPath {
            package_name,
            asset_name,
        }
    }

    /// Read a `TopLevelAssetPath` from an asset
    pub fn read<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let package_name = match asset.get_object_version_ue5()
            >= ObjectVersionUE5::FSOFTOBJECTPATH_REMOVE_ASSET_PATH_FNAMES
        {
            true => Some(asset.read_fname()?),
            false => None,
        };
        let asset_name = asset.read_fname()?;

        Ok(TopLevelAssetPath {
            package_name,
            asset_name,
        })
    }

    /// Write a `TopLevelAssetPath` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        if asset.get_object_version_ue5()
            >= ObjectVersionUE5::FSOFTOBJECTPATH_REMOVE_ASSET_PATH_FNAMES
        {
            let Some(package_name) = self.package_name.as_ref() else {
                return Err(Error::no_data("ObjectVersionUE5 is >= FSOFTOBJECTPATH_REMOVE_ASSET_PATH_FNAMES, but package_name is None".to_string()));
            };

            asset.write_fname(package_name)?;
        }

        asset.write_fname(&self.asset_name)?;

        Ok(())
    }
}

/// Soft object path
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SoftObjectPath {
    /// Asset path
    pub asset_path: TopLevelAssetPath,
    /// Sub path string
    pub sub_path_string: Option<String>,
}

impl SoftObjectPath {
    /// Read a `SoftObjectPath` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let asset_path = TopLevelAssetPath::read(asset)?;
        let sub_path_string = asset.read_fstring()?;

        Ok(SoftObjectPath {
            asset_path,
            sub_path_string,
        })
    }

    /// Write a `SoftObjectPath` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.asset_path.write(asset)?;
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
