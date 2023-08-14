//! IoStore exports

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
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
pub struct ExportMapEntry {
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

impl ExportMapEntry {
    /// Read `ExportMapEntry` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
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

        Ok(ExportMapEntry {
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

    /// Write `ExportMapEntry` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
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
}
