use crate::error::Error;
use crate::exports::unknown_export::UnknownExport;
use crate::exports::{ExportTrait, ExportUnknownTrait};
use crate::Asset;
use std::io::{Cursor, Read, Write};

use super::ExportNormalTrait;

#[derive(Clone)]
pub struct RawExport {
    pub unknown_export: UnknownExport,

    pub data: Vec<u8>,
}

impl ExportNormalTrait for RawExport {
    fn get_normal_export<'a>(&'a self) -> Option<&'a super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportUnknownTrait for RawExport {
    fn get_unknown_export<'a>(&'a self) -> &'a UnknownExport {
        &self.unknown_export
    }

    fn get_unknown_export_mut<'a>(&'a mut self) -> &'a mut UnknownExport {
        &mut self.unknown_export
    }
}

impl RawExport {
    pub fn from_unk(unk: UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let cursor = &mut asset.cursor;
        let mut data = vec![0u8; unk.serial_size as usize];
        cursor.read_exact(&mut data)?;

        Ok(RawExport {
            unknown_export: unk,
            data,
        })
    }
}

impl ExportTrait for RawExport {
    fn write(&self, _asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        cursor.write(&self.data)?;
        Ok(())
    }
}
