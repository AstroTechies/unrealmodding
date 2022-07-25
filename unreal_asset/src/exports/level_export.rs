use super::ExportBaseTrait;
use super::ExportNormalTrait;
use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::cursor_ext::CursorExt;
use crate::error::Error;
use crate::exports::base_export::BaseExport;
use crate::exports::normal_export::NormalExport;
use crate::exports::ExportTrait;
use crate::implement_get;
use crate::unreal_types::NamespacedString;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Cursor;

#[derive(Clone)]
pub struct LevelExport {
    pub normal_export: NormalExport,

    pub index_data: Vec<i32>,
    pub level_type: NamespacedString,
    pub flags_probably: u64,
    pub misc_category_data: Vec<i32>,
}

implement_get!(LevelExport);

impl LevelExport {
    pub fn from_base<Reader: AssetReader>(
        unk: &BaseExport,
        asset: &mut Reader,
        next_starting: u64,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(unk, asset)?;

        asset.read_i32::<LittleEndian>()?;

        let num_index_entries = asset.read_i32::<LittleEndian>()?;
        let mut index_data = Vec::with_capacity(num_index_entries as usize);
        for _i in 0..num_index_entries as usize {
            index_data.push(asset.read_i32::<LittleEndian>()?);
        }

        let nms = asset.read_string()?;
        asset.read_i32::<LittleEndian>()?; // null
        let val = asset.read_string()?;
        let level_type = NamespacedString::new(nms, val);

        asset.read_i64::<LittleEndian>()?; // null
        let flags_probably = asset.read_u64::<LittleEndian>()?;
        let mut misc_category_data = Vec::new();
        while asset.position() < next_starting - 1 {
            misc_category_data.push(asset.read_i32::<LittleEndian>()?);
        }
        asset.read_exact(&mut [0u8; 1])?;

        Ok(LevelExport {
            normal_export,
            index_data,
            level_type,
            flags_probably,
            misc_category_data,
        })
    }
}

impl ExportTrait for LevelExport {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;

        cursor.write_i32::<LittleEndian>(0)?;
        cursor.write_i32::<LittleEndian>(self.index_data.len() as i32)?;
        for index in &self.index_data {
            cursor.write_i32::<LittleEndian>(*index)?;
        }

        cursor.write_string(&self.level_type.namespace)?;
        cursor.write_i32::<LittleEndian>(0)?;
        cursor.write_string(&self.level_type.value)?;

        cursor.write_u64::<LittleEndian>(0)?;
        cursor.write_u64::<LittleEndian>(self.flags_probably)?;

        for data in &self.misc_category_data {
            cursor.write_i32::<LittleEndian>(*data)?;
        }
        cursor.write_u8(0)?;
        Ok(())
    }
}
