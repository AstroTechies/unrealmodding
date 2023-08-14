//! IoStore zen-specific implementations

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use unreal_asset_base::{
    custom_version::CustomVersion,
    engine_version::EngineVersion,
    enums::ECustomVersionSerializationFormat,
    error::Error,
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::PackageIndexTrait,
};

use crate::iostore::{enums::EZenPackageVersion, FMappedName};

/// Zen package versioning info
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZenPackageVersioningInfo {
    /// Zen version
    pub zen_version: EZenPackageVersion,
    /// Object version
    pub object_version: ObjectVersion,
    /// Object version ue5
    pub object_version_ue5: ObjectVersionUE5,
    /// File licensee version
    pub file_licensee_version: i32,
    /// Custom versions
    pub custom_versions: Vec<CustomVersion>,
}

impl ZenPackageVersioningInfo {
    /// Read `ZenPackageVersioningInfo` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let zen_version = EZenPackageVersion::try_from(archive.read_u32::<LE>()?)?;

        let object_version = ObjectVersion::try_from(archive.read_i32::<LE>()?)?;
        let object_version_ue5 = ObjectVersionUE5::try_from(archive.read_i32::<LE>()?)?;

        let file_licensee_version = archive.read_i32::<LE>()?;
        let custom_versions = archive
            .read_custom_version_container(ECustomVersionSerializationFormat::Optimized, None)?;

        Ok(ZenPackageVersioningInfo {
            zen_version,
            object_version,
            object_version_ue5,
            file_licensee_version,
            custom_versions,
        })
    }

    /// Write `ZenPackageVersioningInfo` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u32::<LE>(self.zen_version as u32)?;
        archive.write_i32::<LE>(self.object_version as i32)?;
        archive.write_i32::<LE>(self.object_version_ue5 as i32)?;
        archive.write_i32::<LE>(self.file_licensee_version)?;

        archive.write_custom_version_container(
            ECustomVersionSerializationFormat::Optimized,
            &self.custom_versions,
        )?;

        Ok(())
    }
}

/// Zen package summary
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ZenPackageSummary {
    /// Has versioning info
    pub has_versioning_info: bool,
    /// Header size
    pub header_size: u32,
    /// Name
    pub name: FMappedName,
    /// Package flags
    pub package_flags: EPackageFlags,
    /// Cooked header size
    pub cooked_header_size: u32,
    /// Imported public export hashes offset
    pub imported_public_export_hashes_offset: i32,
    /// Import map offset
    pub import_map_offset: i32,
    /// Export map offset
    pub export_map_offset: i32,
    /// Export bundle entries offset
    pub export_bundle_entries_offset: i32,
    /// Graph data offset
    pub graph_data_offset: Option<i32>,
    /// Dependency bundle headers offset
    pub dependency_bundle_headers_offset: Option<i32>,
    /// Dependency bundle entries offset
    pub dependency_bundle_entries_offset: Option<i32>,
}

impl ZenPackageSummary {
    /// Read `ZenPackageSummary` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let has_versioning_info = archive.read_u32::<LE>()? > 0;
        let header_size = archive.read_u32::<LE>()?;
        let name = FMappedName::read(archive)?;
        let package_flags = EPackageFlags::from_bits_retain(archive.read_u32::<LE>()?);
        let cooked_header_size = archive.read_u32::<LE>()?;
        let imported_public_export_hashes_offset = archive.read_i32::<LE>()?;
        let import_map_offset = archive.read_i32::<LE>()?;
        let export_map_offset = archive.read_i32::<LE>()?;
        let export_bundle_entries_offset = archive.read_i32::<LE>()?;

        let (graph_data_offset, dependency_bundle_headers_offset, dependency_bundle_entries_offset) =
            match archive.get_engine_version() >= EngineVersion::VER_UE5_3 {
                true => {
                    let headers = archive.read_i32::<LE>()?;
                    let entries = archive.read_i32::<LE>()?;
                    (None, Some(headers), Some(entries))
                }
                false => {
                    let graph_data = archive.read_i32::<LE>()?;
                    (Some(graph_data), None, None)
                }
            };

        Ok(ZenPackageSummary {
            has_versioning_info,
            header_size,
            name,
            package_flags,
            cooked_header_size,
            imported_public_export_hashes_offset,
            import_map_offset,
            export_map_offset,
            export_bundle_entries_offset,
            graph_data_offset,
            dependency_bundle_headers_offset,
            dependency_bundle_entries_offset,
        })
    }

    /// Write `ZenPackageSummary` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u32::<LE>(match self.has_versioning_info {
            true => 1,
            false => 0,
        })?;

        archive.write_u32::<LE>(self.header_size)?;
        self.name.write(archive)?;
        archive.write_u32::<LE>(self.package_flags.bits())?;
        archive.write_u32::<LE>(self.cooked_header_size)?;
        archive.write_i32::<LE>(self.imported_public_export_hashes_offset)?;
        archive.write_i32::<LE>(self.import_map_offset)?;
        archive.write_i32::<LE>(self.export_map_offset)?;
        archive.write_i32::<LE>(self.export_bundle_entries_offset)?;

        if archive.get_engine_version() >= EngineVersion::VER_UE5_3 {
            archive.write_i32::<LE>(self.dependency_bundle_headers_offset.ok_or_else(|| {
                Error::no_data(
                    "engine_version >= 5.3 but dependency_bundle_headers_offset is None"
                        .to_string(),
                )
            })?)?;
            archive.write_i32::<LE>(self.dependency_bundle_entries_offset.ok_or_else(|| {
                Error::no_data(
                    "engine_version >= 5.3 but dependency_bundle_entries_offset is None"
                        .to_string(),
                )
            })?)?;
        } else {
            archive.write_i32::<LE>(self.graph_data_offset.ok_or_else(|| {
                Error::no_data("engine_version < 5.3 but graph_data_offset is None".to_string())
            })?)?;
        }

        Ok(())
    }
}
