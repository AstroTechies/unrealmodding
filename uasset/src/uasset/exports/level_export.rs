use std::io::{Cursor, Error, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::unreal_types::{FName, Guid, NamespacedString};


#[derive(Debug, Default)]
pub struct LevelExport {
    normal_export: NormalExport,

    index_data: Vec<i32>,
    level_type: NamespacedString,
    flags_probably: u64,
    misc_category_data: Vec<i32>
}

impl LevelExport {
    pub fn from_unk(unk: &UnknownExport, cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset, next_starting: i32) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, cursor, asset)?;

        cursor.read_i32::<LittleEndian>()?;

        let num_index_entries = cursor.read_i32::<LittleEndian>()?;
        let mut index_data = Vec::with_capacity(num_index_entries as usize);
        for i in 0..num_index_entries as usize {
            index_data[i] = cursor.read_i32::<LittleEndian>()?;
        }

        let nms = cursor.read_string()?;
        cursor.read_i32::<LittleEndian>()?; // null
        let val = cursor.read_string()?;
        let level_type = NamespacedString::new(nms, val);

        cursor.read_i64::<LittleEndian>()?; // null
        let flags_probably = cursor.read_u64::<LittleEndian>()?;
        let mut misc_category_data = Vec::new();
        while cursor.position() < next_starting as u64 - 1 {
            misc_category_data.push(cursor.read_i32::<LittleEndian>()?);
        }
        cursor.read_exact(&mut [0u8; 1])?;

        Ok(LevelExport {
            normal_export,
            index_data,
            level_type,
            flags_probably,
            misc_category_data
        })
    }
}