use std::collections::HashMap;

use byteorder::LittleEndian;

use crate::{
    custom_version::FAssetRegistryVersionType,
    error::Error,
    flags::EPackageFlags,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::FName,
};

use super::asset_bundle_data::AssetBundleData;

#[derive(Clone, Debug)]
pub struct TopLevelAssetPath {
    pub package_name: FName,
    pub asset_name: FName,
}

impl TopLevelAssetPath {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let package_name = asset.read_fname()?;
        let asset_name = asset.read_fname()?;

        Ok(Self {
            package_name,
            asset_name,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_fname(&self.package_name)?;
        writer.write_fname(&self.asset_name)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AssetData {
    pub object_path: FName,
    pub package_name: FName,
    pub package_path: FName,
    pub asset_name: FName,

    pub asset_class: Option<FName>,
    pub asset_path: Option<TopLevelAssetPath>,

    pub tags_and_values: HashMap<FName, Option<String>>,
    pub tagged_asset_bundles: AssetBundleData,
    pub chunk_ids: Vec<i32>,
    pub package_flags: EPackageFlags,

    version: FAssetRegistryVersionType,
}

impl AssetData {
    fn read_tags<Reader: AssetReader>(
        asset: &mut Reader,
    ) -> Result<HashMap<FName, Option<String>>, Error> {
        let size = asset.read_i32::<LittleEndian>()?;
        let mut tags_and_values = HashMap::new();

        for _ in 0..size {
            tags_and_values.insert(asset.read_fname()?, asset.read_string()?);
        }
        Ok(tags_and_values)
    }

    fn write_tags<Writer: AssetWriter>(
        asset: &mut Writer,
        tags_and_values: &HashMap<FName, Option<String>>,
    ) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(tags_and_values.len() as i32)?;
        for (key, value) in tags_and_values {
            asset.write_fname(key)?;
            asset.write_string(value)?;
        }
        Ok(())
    }

    pub fn new<Reader: AssetReader>(
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
        let chunk_ids =
            asset.read_array(|asset: &mut Reader| Ok(asset.read_i32::<LittleEndian>()?))?; // if we don't explicitly specify the type inside lambda the compiler will crashd
        let package_flags = EPackageFlags::from_bits(asset.read_u32::<LittleEndian>()?)
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

    #[allow(clippy::too_many_arguments)]
    pub fn from_data(
        object_path: FName,
        package_name: FName,
        package_path: FName,
        asset_name: FName,
        asset_class: Option<FName>,
        asset_path: Option<TopLevelAssetPath>,
        tags_and_values: HashMap<FName, Option<String>>,
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

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
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

        writer.write_i32::<LittleEndian>(self.chunk_ids.len() as i32)?;
        for chunk_id in &self.chunk_ids {
            writer.write_i32::<LittleEndian>(*chunk_id)?;
        }

        writer.write_u32::<LittleEndian>(self.package_flags.bits())?;
        Ok(())
    }
}
