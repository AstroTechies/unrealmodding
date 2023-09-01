//! IoStore container header

use std::io::{Cursor, Read, Seek};

use byteorder::{ReadBytesExt, LE};
use unreal_asset_base::{
    containers::{chain::Chain, name_map::NameMap},
    engine_version::{get_object_versions, EngineVersion},
    enums::EIoContainerHeaderVersion,
    error::{Error, IoStoreError},
    reader::{archive_reader::ArchiveReader, raw_reader::RawReader, ArchiveWriter},
    types::PackageIndexTrait,
};
use unreal_helpers::UnrealWriteExt;

use super::{
    cas::reader::IoStoreReader,
    name::{FMappedName, FNameEntrySerialized},
    package_id::PackageId,
    package_store_entry::FilePackageStoreEntry,
    providers::IoStoreProvider,
    toc::{
        chunk::{EIoChunkType, EIoChunkType5},
        IoContainerId,
    },
    PackageObjectIndex,
};

/// IoStore segment info
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoSegmentInfo {
    /// Package ids
    pub package_ids: Vec<PackageId>,
    /// Entries
    pub entries: Vec<FilePackageStoreEntry>,
}

impl IoSegmentInfo {
    /// Read `IoSegmentInfo` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
        version: EIoContainerHeaderVersion,
    ) -> Result<Self, Error> {
        let package_ids = archive.read_array(PackageId::read)?;
        let entries = archive.read_array(|reader| FilePackageStoreEntry::read(reader, version))?;

        Ok(IoSegmentInfo {
            package_ids,
            entries,
        })
    }

    /// Write `IoSegmentInfo` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_array(&self.package_ids, |writer, id| id.write(writer))?;
        unimplemented!();
    }
}

/// IoStore container header localized package
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoContainerHeaderLocalizedPackage {
    /// Source package id
    pub source_package_id: PackageId,
    /// Source package name
    pub source_package_name: FMappedName,
}

impl IoContainerHeaderLocalizedPackage {
    /// Read `IoContainerHeaderLocalizedPackage` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let source_package_id = PackageId::read(archive)?;
        let source_package_name = FMappedName::read(archive)?;

        Ok(IoContainerHeaderLocalizedPackage {
            source_package_id,
            source_package_name,
        })
    }

    /// Write `IoContainerHeaderLocalizedPackage` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        self.source_package_id.write(archive)?;
        self.source_package_name.write(archive)?;

        Ok(())
    }
}

/// IoStore container header package redirect
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoContainerHeaderPackageRedirect {
    /// Source package id
    pub source_package_id: PackageId,
    /// Target package id
    pub target_package_id: PackageId,
    /// Source package name
    pub source_package_name: FMappedName,
}

impl IoContainerHeaderPackageRedirect {
    /// Read `IoContainerHeaderPackageRedirect` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let source_package_id = PackageId::read(archive)?;
        let target_package_id = PackageId::read(archive)?;
        let source_package_name = FMappedName::read(archive)?;

        Ok(IoContainerHeaderPackageRedirect {
            source_package_id,
            target_package_id,
            source_package_name,
        })
    }

    /// Write `IoContainerHeaderPackageRedirect` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        self.source_package_id.write(archive)?;
        self.target_package_id.write(archive)?;
        self.source_package_name.write(archive)?;

        Ok(())
    }
}

/// IoStore container header
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoContainerHeader {
    /// Version
    pub version: EIoContainerHeaderVersion,
    /// Container id
    pub container_id: IoContainerId,
    /// Package count
    pub package_count: Option<u32>,
    /// Main segment info
    pub main_segment: IoSegmentInfo,
    /// Optional segment info
    pub optional_segment: Option<IoSegmentInfo>,
    /// Container name map
    pub container_name_map: Vec<FNameEntrySerialized>,
    /// Localized packages
    pub localized_packages: Option<Vec<IoContainerHeaderLocalizedPackage>>,
    /// Package redirects
    pub package_redirects: Vec<IoContainerHeaderPackageRedirect>,
}

impl IoContainerHeader {
    const MAGIC: u32 = 0x496f436e;

    /// Read `IoContainerHeader` from an [`IoStoreReader`]
    pub fn new<R: Read + Seek, P: IoStoreProvider<R>>(
        reader: &mut IoStoreReader<R, P>,
        engine_version: EngineVersion,
    ) -> Result<Self, Error> {
        let chunk_type = match engine_version >= EngineVersion::VER_UE5_0 {
            true => EIoChunkType5::ContainerHeader as u8,
            false => EIoChunkType::ContainerHeader as u8,
        };

        let header_chunk = reader
            .toc_resource
            .get_chunk_offset_by_type(chunk_type)?
            .ok_or_else(|| IoStoreError::no_chunk("ContainerHeader"))?;

        let mut chunk_data = vec![0u8; header_chunk.length as usize];
        reader.read_all(header_chunk.offset, &mut chunk_data)?;

        let (object_version, object_version_ue5) = get_object_versions(engine_version);
        let mut archive = RawReader::<PackageObjectIndex, _>::new(
            Chain::new(Cursor::new(chunk_data), None),
            object_version,
            object_version_ue5,
            false,
            NameMap::new(),
        );
        Self::read(&mut archive)
    }

    /// Read `IoContainerHeader` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let magic = archive.read_u32::<LE>()?;
        if magic != Self::MAGIC {
            return Err(Error::invalid_file("Header is invalid".to_string()));
        }

        let version = EIoContainerHeaderVersion::try_from(archive.read_u32::<LE>()?)?;

        let container_id = IoContainerId::read(archive)?;

        let package_count = match version < EIoContainerHeaderVersion::OptionalSegmentPackages {
            true => Some(archive.read_u32::<LE>()?),
            false => None,
        };

        let main_segment = IoSegmentInfo::read(archive, version)?;

        let optional_segment = match version >= EIoContainerHeaderVersion::OptionalSegmentPackages {
            true => Some(IoSegmentInfo::read(archive, version)?),
            false => None,
        };

        let container_name_map = FNameEntrySerialized::read_name_batch(archive)?;

        let localized_packages = match version >= EIoContainerHeaderVersion::LocalizedPackages {
            true => Some(archive.read_array(IoContainerHeaderLocalizedPackage::read)?),
            false => None,
        };

        let package_redirects = archive.read_array(IoContainerHeaderPackageRedirect::read)?;

        Ok(IoContainerHeader {
            version,
            container_id,
            package_count,
            main_segment,
            optional_segment,
            container_name_map,
            localized_packages,
            package_redirects,
        })
    }
}
