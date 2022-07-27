use byteorder::LittleEndian;

use crate::{
    error::Error,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
};

#[derive(Debug)]
pub struct FMD5Hash {
    pub hash: Option<[u8; 16]>,
}

impl FMD5Hash {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut hash = None;

        let has_hash = asset.read_u32::<LittleEndian>()?;
        if has_hash != 0 {
            let mut hash_data = [0u8; 16];
            asset.read_exact(&mut hash_data)?;
            hash = Some(hash_data);
        }

        Ok(Self { hash })
    }

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        if let Some(hash) = &self.hash {
            writer.write_u32::<LittleEndian>(1)?;
            writer.write_all(hash)?;
        } else {
            writer.write_u32::<LittleEndian>(0)?;
        }
        Ok(())
    }
}
