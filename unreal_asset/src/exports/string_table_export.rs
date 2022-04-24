use crate::cursor_ext::CursorExt;
use crate::error::Error;
use crate::exports::normal_export::NormalExport;
use crate::exports::unknown_export::UnknownExport;
use crate::exports::ExportTrait;
use crate::implement_get;
use crate::unreal_types::StringTable;
use crate::Asset;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use super::ExportNormalTrait;
use super::ExportUnknownTrait;

#[derive(Clone)]
pub struct StringTableExport {
    normal_export: NormalExport,

    table: StringTable,
}

implement_get!(StringTableExport);

impl StringTableExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;
        asset.cursor.read_i32::<LittleEndian>()?;

        let mut table = StringTable::new(asset.cursor.read_string()?);

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        for _i in 0..num_entries {
            table.value.insert(
                asset
                    .cursor
                    .read_string()?
                    .ok_or(Error::no_data("StringTable key is None".to_string()))?,
                asset
                    .cursor
                    .read_string()?
                    .ok_or(Error::no_data("StringTable value is None".to_string()))?,
            );
        }

        Ok(StringTableExport {
            normal_export,
            table,
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
            cursor.write_string(&Some(key.clone()))?;
            cursor.write_string(&Some(value.clone()))?;
        }
        Ok(())
    }
}
