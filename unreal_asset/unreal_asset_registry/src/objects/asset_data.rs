//! Asset bundle asset data

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    containers::IndexedMap,
    custom_version::FAssetRegistryVersionType,
    flags::EPackageFlags,
    reader::{ArchiveReader, ArchiveWriter},
    types::{FName, PackageIndexTrait},
    Error,
};

use crate::objects::asset_bundle_data::AssetBundleData;

/// Top level asset path
#[derive(Clone, Debug)]
pub struct TopLevelAssetPath {
    /// Package name
    pub package_name: FName,
    /// Asset name
    pub asset_name: FName,
}

impl TopLevelAssetPath {
    /// Read a `TopLevelAssetPath` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let package_name = asset.read_fname()?;
        let asset_name = asset.read_fname()?;

        Ok(Self {
            package_name,
            asset_name,
        })
    }

    /// Write a `TopLevelAssetPath` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        writer.write_fname(&self.package_name)?;
        writer.write_fname(&self.asset_name)?;
        Ok(())
    }
}

/// Asset data
#[derive(Debug, Clone)]
pub struct AssetData {
    /// Object path
    pub object_path: FName,
    /// Package name
    pub package_name: FName,
    /// Package path
    pub package_path: FName,
    /// Asset name
    pub asset_name: FName,

    /// Asset class
    pub asset_class: Option<FName>,
    /// Asset path
    pub asset_path: Option<TopLevelAssetPath>,

    /// Tags and values
    pub tags_and_values: IndexedMap<FName, Option<String>>,
    /// Tagged asset bundles
    pub tagged_asset_bundles: AssetBundleData,
    /// Chunk ids
    pub chunk_ids: Vec<i32>,
    /// Package flags
    pub package_flags: EPackageFlags,

    /// Asset registry version
    version: FAssetRegistryVersionType,
}

impl AssetData {
    /// Read `AssetData` tags
    fn read_tags<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<IndexedMap<FName, Option<String>>, Error> {
        let size = asset.read_i32::<LE>()?;
        let mut tags_and_values = IndexedMap::new();

        for _ in 0..size {
            tags_and_values.insert(asset.read_fname()?, asset.read_fstring()?);
        }
        Ok(tags_and_values)
    }

    /// Write `AssetData` tags
    fn write_tags<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        asset: &mut Writer,
        tags_and_values: &IndexedMap<FName, Option<String>>,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(tags_and_values.len() as i32)?;
        for (_, key, value) in tags_and_values {
            asset.write_fname(key)?;
            asset.write_fstring(value.as_deref())?;
        }
        Ok(())
    }

    /// Read `AssetData` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        version: FAssetRegistryVersionType,
    ) -> Result<Self, Error> {
        let object_path = asset.read_fname()?;
        let package_path = asset.read_fname()?;

        let (asset_class, asset_path) = match version >= FAssetRegistryVersionType::ClassPaths {
            true => (None, Some(TopLevelAssetPath::new(asset)?)),
            false => (Some(asset.read_fname()?), None),
        };

        let package_name = asset.read_fname()?;
        let asset_name = asset.read_fname()?;
        let tags = Self::read_tags(asset)?;
        let chunk_ids = asset.read_array(|asset: &mut Reader| Ok(asset.read_i32::<LE>()?))?; // if we don't explicitly specify the type inside the lambda the compiler will crash
        let package_flags = EPackageFlags::from_bits(asset.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid package flags".to_string()))?;

        Ok(Self {
            object_path,
            package_name,
            package_path,
            asset_name,
            asset_class,
            asset_path,
            tags_and_values: tags,
            tagged_asset_bundles: AssetBundleData::default(),
            chunk_ids,
            package_flags,

            version,
        })
    }

    /// Create a new `AssetData` instance
    #[allow(clippy::too_many_arguments)]
    pub fn from_data(
        object_path: FName,
        package_name: FName,
        package_path: FName,
        asset_name: FName,
        asset_class: Option<FName>,
        asset_path: Option<TopLevelAssetPath>,
        tags_and_values: IndexedMap<FName, Option<String>>,
        tagged_asset_bundles: AssetBundleData,
        chunk_ids: Vec<i32>,
        package_flags: EPackageFlags,

        version: FAssetRegistryVersionType,
    ) -> Self {
        Self {
            object_path,
            package_name,
            package_path,
            asset_name,
            asset_class,
            asset_path,
            tags_and_values,
            tagged_asset_bundles,
            chunk_ids,
            package_flags,

            version,
        }
    }

    /// Write `AssetData` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        writer.write_fname(&self.object_path)?;
        writer.write_fname(&self.package_path)?;

        match self.version >= FAssetRegistryVersionType::ClassPaths {
            true => {
                if self.asset_path.is_none() {
                    return Err(Error::invalid_file(
                        "Asset path is None for a file with version >= ClassPaths".to_string(),
                    ));
                }
                self.asset_path.as_ref().unwrap().write(writer)?;
            }
            false => {
                if self.asset_class.is_none() {
                    return Err(Error::invalid_file(
                        "Asset class is None for a file with version < ClassPaths".to_string(),
                    ));
                }
                writer.write_fname(self.asset_class.as_ref().unwrap())?;
            }
        };

        writer.write_fname(&self.package_name)?;
        writer.write_fname(&self.asset_name)?;
        Self::write_tags(writer, &self.tags_and_values)?;

        writer.write_i32::<LE>(self.chunk_ids.len() as i32)?;
        for chunk_id in &self.chunk_ids {
            writer.write_i32::<LE>(*chunk_id)?;
        }

        writer.write_u32::<LE>(self.package_flags.bits())?;
        Ok(())
    }
}
