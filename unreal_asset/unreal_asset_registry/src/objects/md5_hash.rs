//! MD5 hash
//!
use byteorder::LE;

use unreal_asset_base::{
    reader::{ArchiveReader, ArchiveWriter},
    Error,
};

/// Unreal MD5 hash
#[derive(Debug)]
pub struct FMD5Hash {
    /// Hash
    pub hash: Option<[u8; 16]>,
}

impl FMD5Hash {
    /// Read a `FMD5Hash` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut hash = None;

        let has_hash = asset.read_u32::<LE>()?;
        if has_hash != 0 {
            let mut hash_data = [0u8; 16];
            asset.read_exact(&mut hash_data)?;
            hash = Some(hash_data);
        }

        Ok(Self { hash })
    }

    /// Write a `FMD5Hash` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        if let Some(hash) = &self.hash {
            writer.write_u32::<LE>(1)?;
            writer.write_all(hash)?;
        } else {
            writer.write_u32::<LE>(0)?;
        }
        Ok(())
    }
}
