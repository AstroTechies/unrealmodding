use std::io::{Cursor, Error, Read};
use crate::uasset::Asset;
use crate::uasset::exports::unknown_export::UnknownExport;

pub struct RawExport {
    unknown_export: UnknownExport,

    data: Vec<u8>
}

impl RawExport {
    pub fn from_unk(unk: UnknownExport, cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let mut data = Vec::with_capacity(unk.serial_size as usize);
        cursor.read_exact(&mut data)?;

        Ok(RawExport {
            unknown_export: unk,
            data
        })
    }
}