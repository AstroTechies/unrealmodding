use byteorder::LittleEndian;

use crate::error::Error;
use crate::properties::{soft_path_property::SoftObjectPathProperty, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::FName;

#[derive(Debug, Clone)]
pub struct AssetBundleEntry {
    pub bundle_name: FName,
    pub bundle_assets: Vec<SoftObjectPathProperty>,
}

impl AssetBundleEntry {
    pub fn new<Reader>(asset: &mut Reader) -> Result<Self, Error>
    where
        Reader: AssetReader,
    {
        let bundle_name = asset.read_fname()?;
        let bundle_assets = asset.read_array(|asset: &mut Reader| {
            SoftObjectPathProperty::new(asset, FName::from_slice("None"), false, 0, 0)
        })?;

        Ok(Self {
            bundle_name,
            bundle_assets,
        })
    }

    pub fn from_data(bundle_name: FName, bundle_assets: Vec<SoftObjectPathProperty>) -> Self {
        Self {
            bundle_name,
            bundle_assets,
        }
    }

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_fname(&self.bundle_name)?;

        writer.write_i32::<LittleEndian>(self.bundle_assets.len() as i32)?;

        for bundle_asset in &self.bundle_assets {
            bundle_asset.write(writer, false)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AssetBundleData {
    bundles: Vec<AssetBundleEntry>,
}

impl AssetBundleData {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let bundles = asset.read_array(|asset: &mut Reader| AssetBundleEntry::new(asset))?;

        Ok(Self { bundles })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.bundles.len() as i32)?;

        for bundle in &self.bundles {
            bundle.write(asset)?;
        }

        Ok(())
    }

    pub fn from_data(bundles: Vec<AssetBundleEntry>) -> Self {
        Self { bundles }
    }
}
