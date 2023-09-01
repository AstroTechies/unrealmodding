//! IoStore package store entry

use std::io::SeekFrom;

use byteorder::{ReadBytesExt, LE};
use unreal_asset_base::{
    enums::EIoContainerHeaderVersion,
    reader::ArchiveReader,
    types::{sha::FShaHash, PackageIndexTrait},
    Error,
};

use super::package_id::PackageId;

/// IoStore package store entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePackageStoreEntry {
    /// Export count
    pub export_count: Option<i32>,
    /// Export bundle count
    pub export_bundle_count: Option<i32>,
    /// Imported packages
    pub imported_packages: Vec<PackageId>,
    /// Shader map hashes
    pub shader_map_hashes: Vec<FShaHash>,
}

impl FilePackageStoreEntry {
    /// Read `FilePackageStoreEntry` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
        version: EIoContainerHeaderVersion,
    ) -> Result<Self, Error> {
        let (export_count, export_bundle_count) =
            match version < EIoContainerHeaderVersion::NoExportInfo {
                true => {
                    let export_count = archive.read_i32::<LE>()?;
                    let export_bundle_count = archive.read_i32::<LE>()?;
                    (Some(export_count), Some(export_bundle_count))
                }
                false => (None, None),
            };

        let imported_packages = Self::read_carrayview(archive, PackageId::read)?;
        let shader_map_hashes = Self::read_carrayview(archive, FShaHash::read)?;

        Ok(FilePackageStoreEntry {
            export_count,
            export_bundle_count,
            imported_packages,
            shader_map_hashes,
        })
    }

    /// Read a `TFilePackageStoreEntryCArrayView` from an archive
    pub fn read_carrayview<R: ArchiveReader<impl PackageIndexTrait>, T>(
        archive: &mut R,
        f: impl Fn(&mut R) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let start_pos = archive.position();

        let length = archive.read_i32::<LE>()?;
        let offset_to_data = archive.read_i32::<LE>()?;

        let next_pos = archive.position();

        archive.seek(SeekFrom::Start(start_pos + offset_to_data as u64))?;

        let arr = archive.read_array_with_length(length, f)?;

        archive.seek(SeekFrom::Start(next_pos))?;

        Ok(arr)
    }
}
