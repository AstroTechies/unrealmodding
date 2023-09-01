//! Sha hash

use crate::{
    reader::{ArchiveReader, ArchiveWriter},
    Error,
};

use super::PackageIndexTrait;

/// Sha hash
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FShaHash([u8; 20]);

impl FShaHash {
    /// Read `FShaHash` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let mut hash = [0u8; 20];
        archive.read_exact(&mut hash)?;

        Ok(FShaHash(hash))
    }

    /// Write `FShaHash` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_all(&self.0)?;

        Ok(())
    }
}
