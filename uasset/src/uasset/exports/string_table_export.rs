use std::collections::HashMap;
use std::io::{Cursor, Error};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::unreal_types::StringTable;

use super::ExportNormalTrait;

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
        for i in 0..num_entries {
            table.value.insert(asset.cursor.read_string()?, asset.cursor.read_string()?);
        }

        Ok(StringTableExport {
            normal_export,
            table
        })
    }
}