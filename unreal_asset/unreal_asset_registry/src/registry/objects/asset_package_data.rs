//! Asset bundle asset package data

use byteorder::LE;

use unreal_helpers::Guid;

use crate::custom_version::{CustomVersion, FAssetRegistryVersionType};
use crate::error::{Error, RegistryError};
use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::registry::objects::md5_hash::FMD5Hash;
use crate::types::FName;

/// Asset package data
#[derive(Debug)]
pub struct AssetPackageData {
    /// Package name
    pub package_name: FName,
    /// Package guid
    pub package_guid: Guid,
    /// Cooked hash
    pub cooked_hash: Option<FMD5Hash>,
    /// Imported classes
    pub imported_classes: Option<Vec<FName>>,
    /// Size on disk
    pub disk_size: i64,
    /// File version
    pub file_version: i32,
    /// UE5 file version
    pub ue5_version: Option<i32>,
    /// File version licensee
    pub file_version_licensee_ue: i32,
    /// Custom versions
    pub custom_versions: Option<Vec<CustomVersion>>,
    /// Flags
    pub flags: u32,

    /// Asset registry version
    version: FAssetRegistryVersionType,
}

impl AssetPackageData {
    /// Read `AssetPackageData` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        version: FAssetRegistryVersionType,
    ) -> Result<Self, Error> {
        let package_name = asset.read_fname()?;
        let disk_size = asset.read_i64::<LE>()?;

        let package_guid = asset.read_guid()?;

        let mut cooked_hash = None;
        if version >= FAssetRegistryVersionType::AddedCookedMD5Hash {
            cooked_hash = Some(FMD5Hash::new(asset)?);
        }

        let mut file_version = 0;
        let mut ue5_version = None;
        let mut file_version_licensee_ue = -1;
        let mut flags = 0;
        let mut custom_versions = None;
        if version >= FAssetRegistryVersionType::WorkspaceDomain {
            if version >= FAssetRegistryVersionType::PackageFileSummaryVersionChange {
                file_version = asset.read_i32::<LE>()?;
                ue5_version = Some(asset.read_i32::<LE>()?);
            } else {
                file_version = asset.read_i32::<LE>()?;
            }

            file_version_licensee_ue = asset.read_i32::<LE>()?;
            flags = asset.read_u32::<LE>()?;
            custom_versions =
                Some(asset.read_array(|asset: &mut Reader| CustomVersion::read(asset))?);
        }

        let mut imported_classes = None;
        if version >= FAssetRegistryVersionType::PackageImportedClasses {
            imported_classes = Some(asset.read_array(|asset: &mut Reader| asset.read_fname())?);
        }

        Ok(Self {
            package_name,
            package_guid,
            cooked_hash,
            imported_classes,
            disk_size,
            file_version,
            ue5_version,
            file_version_licensee_ue,
            custom_versions,
            flags,

            version,
        })
    }

    /// Write `AssetPackageData` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_fname(&self.package_name)?;
        asset.write_i64::<LE>(self.disk_size)?;
        // TODO change to guid method
        asset.write_all(&self.package_guid.0)?;

        if self.version >= FAssetRegistryVersionType::AddedCookedMD5Hash {
            let cooked_hash = self
                .cooked_hash
                .as_ref()
                .ok_or_else(|| RegistryError::version("Cooked hash".to_string(), self.version))?;

            cooked_hash.write(asset)?;
        }

        if self.version >= FAssetRegistryVersionType::WorkspaceDomain {
            if self.version >= FAssetRegistryVersionType::PackageFileSummaryVersionChange {
                asset.write_i32::<LE>(self.file_version)?;
                asset.write_i32::<LE>(self.ue5_version.ok_or_else(|| {
                    RegistryError::version("UE5 Version".to_string(), self.version)
                })?)?;
            } else {
                asset.write_i32::<LE>(self.file_version)?;
            }

            asset.write_i32::<LE>(self.file_version_licensee_ue)?;
            asset.write_u32::<LE>(self.flags)?;

            let custom_versions = self.custom_versions.as_ref().ok_or_else(|| {
                RegistryError::version("Custom versions".to_string(), self.version)
            })?;

            asset.write_i32::<LE>(custom_versions.len() as i32)?;
            for custom_version in custom_versions {
                custom_version.write(asset)?;
            }
        }

        if self.version >= FAssetRegistryVersionType::PackageImportedClasses {
            let imported_classes = self.imported_classes.as_ref().ok_or_else(|| {
                RegistryError::version("Imported classes".to_string(), self.version)
            })?;

            for immported_class in imported_classes {
                asset.write_fname(immported_class)?;
            }
        }

        Ok(())
    }
}
