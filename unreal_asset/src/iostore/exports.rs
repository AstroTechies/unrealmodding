//! IoStore exports

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_base::{
    engine_version::EngineVersion,
    error::Error,
    flags::EObjectFlags,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::PackageIndexTrait,
};

use super::{flags::EExportFilterFlags, FMappedName, PackageObjectIndex};

/// IoStore export map entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreExportMapEntry {
    /// Cooked serialized offset
    pub cooked_serial_offset: u64,
    /// Cooked serialized size
    pub cooked_serial_size: u64,
    /// Object name
    pub object_name: FMappedName,
    /// Outer index
    pub outer_index: PackageObjectIndex,
    /// Class index
    pub class_index: PackageObjectIndex,
    /// Super index
    pub super_index: PackageObjectIndex,
    /// Template index
    pub template_index: PackageObjectIndex,
    /// Global import index < 5.0 only
    pub global_import_index: Option<PackageObjectIndex>,
    /// Public export hash >= 5.0 only
    pub public_export_hash: Option<u64>,
    /// Object flags
    pub object_flags: EObjectFlags,
    /// Filter flags
    pub filter_flags: EExportFilterFlags,
    /// Padding
    pub padding: [u8; 3],
}

impl IoStoreExportMapEntry {
    /// Read `IoStoreExportMapEntry` from an archive
    pub fn read<R: ArchiveReader<PackageObjectIndex>>(archive: &mut R) -> Result<Self, Error> {
        let cooked_serial_offset = archive.read_u64::<LE>()?;
        let cooked_serial_size = archive.read_u64::<LE>()?;

        let object_name = FMappedName::read(archive)?;

        let outer_index = PackageObjectIndex::read(archive)?;
        let class_index = PackageObjectIndex::read(archive)?;
        let super_index = PackageObjectIndex::read(archive)?;
        let template_index = PackageObjectIndex::read(archive)?;

        let (global_import_index, public_export_hash) =
            match archive.get_engine_version() >= EngineVersion::VER_UE5_0 {
                true => {
                    let hash = archive.read_u64::<LE>()?;
                    (None, Some(hash))
                }
                false => {
                    let index = PackageObjectIndex::read(archive)?;
                    (Some(index), None)
                }
            };

        let object_flags = EObjectFlags::from_bits_retain(archive.read_u32::<LE>()?);
        let filter_flags = EExportFilterFlags::from_bits_retain(archive.read_u8()?);

        let mut padding = [0u8; 3];
        archive.read_exact(&mut padding)?;

        Ok(IoStoreExportMapEntry {
            cooked_serial_offset,
            cooked_serial_size,
            object_name,
            outer_index,
            class_index,
            super_index,
            template_index,
            global_import_index,
            public_export_hash,
            object_flags,
            filter_flags,
            padding,
        })
    }

    /// Write `IoStoreExportMapEntry` to an archive
    pub fn write<W: ArchiveWriter<PackageObjectIndex>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u64::<LE>(self.cooked_serial_offset)?;
        archive.write_u64::<LE>(self.cooked_serial_size)?;

        self.object_name.write(archive)?;

        self.outer_index.write(archive)?;
        self.class_index.write(archive)?;
        self.super_index.write(archive)?;
        self.template_index.write(archive)?;

        if archive.get_engine_version() >= EngineVersion::VER_UE5_0 {
            archive.write_u64::<LE>(self.public_export_hash.ok_or_else(|| {
                Error::no_data("engine_version >= 5.0 but public_export_hash is None".to_string())
            })?)?;
        } else {
            let Some(index) = self.global_import_index else {
                return Err(Error::no_data(
                    "engine_version < 5.0 but global_import_index is None".to_string(),
                ));
            };

            index.write(archive)?;
        }

        archive.write_u32::<LE>(self.object_flags.bits())?;
        archive.write_u8(self.filter_flags.bits())?;

        archive.write_all(&self.padding)?;
        Ok(())
    }

    /// Get `IoStoreExportMapEntry` serialized size
    #[inline(always)]
    pub fn serialized_size() -> u64 {
        72
    }
}

/// IoStore export command type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum EExportCommandType {
    /// Create
    Create,
    /// Serialize
    Serialize,
    /// Enum variant count
    Count,
}

/// IoStore Export bundle entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExportBundleEntry {
    /// Local export index
    pub local_export_index: u32,
    /// Command type
    pub command_type: EExportCommandType,
}

impl ExportBundleEntry {
    /// Read `ExportBundleEntry` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let local_export_index = archive.read_u32::<LE>()?;
        let command_type = EExportCommandType::try_from(archive.read_u32::<LE>()?)?;

        Ok(ExportBundleEntry {
            local_export_index,
            command_type,
        })
    }

    /// Write `ExportBundleEntry` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u32::<LE>(self.local_export_index)?;
        archive.write_u32::<LE>(self.command_type as u32)?;

        Ok(())
    }
}

/// IoStore export bundle header
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExportBundleHeader {
    /// Serialized offset
    pub serial_offset: u64,
    /// First entry index
    pub first_entry_index: u32,
    /// Entry count
    pub entry_count: u32,
}

impl ExportBundleHeader {
    /// Read `ExportBundleHeader` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let serial_offset = archive.read_u64::<LE>()?;
        let first_entry_index = archive.read_u32::<LE>()?;
        let entry_count = archive.read_u32::<LE>()?;

        Ok(ExportBundleHeader {
            serial_offset,
            first_entry_index,
            entry_count,
        })
    }

    /// Write `ExportBundleHeader` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u64::<LE>(self.serial_offset)?;
        archive.write_u32::<LE>(self.first_entry_index)?;
        archive.write_u32::<LE>(self.entry_count)?;

        Ok(())
    }
}
