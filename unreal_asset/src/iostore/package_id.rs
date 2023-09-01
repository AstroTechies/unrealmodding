//! IoStore package id

use std::io;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use unreal_asset_base::{
    crc,
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error,
};

/// Package id
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    /// Id
    pub id: u64,
}

impl PackageId {
    /// Read `PackageId` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let id = archive.read_u64::<LE>()?;
        Ok(PackageId { id })
    }

    /// Write `PackageId` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> io::Result<()> {
        archive.write_u64::<LE>(self.id)?;
        Ok(())
    }

    /// Create a `PackageId` from name
    pub fn from_name(name: &str) -> Self {
        let hash = crc::cityhash64_to_lower(name);
        PackageId { id: hash }
    }
}
