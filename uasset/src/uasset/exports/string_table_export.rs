
use std::io::{Cursor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::implement_get;
use crate::uasset::error::Error;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::exports::ExportTrait;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::unreal_types::StringTable;

use super::ExportNormalTrait;
use super::ExportUnknownTrait;

pub struct StringTableExport {
    normal_export: NormalExport,

    table: StringTable
}

implement_get!(StringTableExport);

impl StringTableExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;
        asset.cursor.read_i32::<LittleEndian>()?;

        let mut table = StringTable::new(asset.cursor.read_string()?);

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        for _i in 0..num_entries {
            table.value.insert(asset.cursor.read_string()?, asset.cursor.read_string()?);
        }

        Ok(StringTableExport {
            normal_export,
            table
        })
    }
}

impl ExportTrait for StringTableExport {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(0)?;

        cursor.write_string(&self.table.namespace)?;
        cursor.write_i32::<LittleEndian>(self.table.value.len() as i32)?;
        for (key, value) in &self.table.value {
            cursor.write_string(key)?;
            cursor.write_string(value)?;
        }
        Ok(())
    }
}