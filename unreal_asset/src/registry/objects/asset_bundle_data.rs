//! Asset registry bundle d ata

use byteorder::LittleEndian;

use crate::error::Error;
use crate::properties::{soft_path_property::SoftObjectPathProperty, PropertyTrait};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::FName;
use crate::unversioned::ancestry::Ancestry;

/// Bundle entry
#[derive(Debug, Clone)]
pub struct AssetBundleEntry {
    /// Bundle name
    pub bundle_name: FName,
    /// Bundle assets
    pub bundle_assets: Vec<SoftObjectPathProperty>,
}

impl AssetBundleEntry {
    /// Read an `AssetBundleEntry` from an asset
    pub fn new<Reader>(asset: &mut Reader) -> Result<Self, Error>
    where
        Reader: ArchiveReader,
    {
        let bundle_name = asset.read_fname()?;
        let bundle_assets = asset.read_array(|asset: &mut Reader| {
            SoftObjectPathProperty::new(
                asset,
                asset.get_name_map().get_mut().add_fname("None"),
                Ancestry::default(),
                false,
                0,
                0,
            )
        })?;

        Ok(Self {
            bundle_name,
            bundle_assets,
        })
    }

    /// Create an `AssetBundleEntry` instance
    pub fn from_data(bundle_name: FName, bundle_assets: Vec<SoftObjectPathProperty>) -> Self {
        Self {
            bundle_name,
            bundle_assets,
        }
    }

    /// Write an `AssetBundleEntry` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_fname(&self.bundle_name)?;

        writer.write_i32::<LittleEndian>(self.bundle_assets.len() as i32)?;

        for bundle_asset in &self.bundle_assets {
            bundle_asset.write(writer, false)?;
        }

        Ok(())
    }
}

/// Bundle data
#[derive(Debug, Default, Clone)]
pub struct AssetBundleData {
    /// Bundles
    bundles: Vec<AssetBundleEntry>,
}

impl AssetBundleData {
    /// Read `AssetBundleData` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let bundles = asset.read_array(|asset: &mut Reader| AssetBundleEntry::new(asset))?;

        Ok(Self { bundles })
    }

    /// Write `AssetBundleData` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.bundles.len() as i32)?;

        for bundle in &self.bundles {
            bundle.write(asset)?;
        }

        Ok(())
    }

    /// Create an `AssetBundleData` instance
    pub fn from_data(bundles: Vec<AssetBundleEntry>) -> Self {
        Self { bundles }
    }
}
