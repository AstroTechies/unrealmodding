use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    io::{self, SeekFrom},
};

use byteorder::LittleEndian;

use crate::{
    custom_version::{CustomVersion, CustomVersionTrait},
    error::Error,
    reader::{asset_trait::AssetTrait, asset_writer::AssetWriter},
    unreal_types::{FName, PackageIndex},
    Import,
};

pub struct NameTableWriter<'name_map, 'writer, Writer: AssetWriter> {
    writer: &'writer mut Writer,

    name_map_lookup: &'name_map HashMap<u64, i32>,
}

impl<'name_map, 'writer, Writer: AssetWriter> NameTableWriter<'name_map, 'writer, Writer> {
    pub fn new(writer: &'writer mut Writer, name_map_lookup: &'name_map HashMap<u64, i32>) -> Self {
        NameTableWriter {
            writer,
            name_map_lookup,
        }
    }
}

impl<'name_map, 'writer, Writer: AssetWriter> AssetTrait
    for NameTableWriter<'name_map, 'writer, Writer>
{
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.writer.get_custom_version::<T>()
    }

    fn position(&self) -> u64 {
        self.writer.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.writer.set_position(pos)
    }

    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        self.writer.seek(style)
    }

    fn get_map_key_override(&self) -> &HashMap<String, String> {
        self.writer.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &HashMap<String, String> {
        self.writer.get_map_value_override()
    }

    fn get_engine_version(&self) -> i32 {
        self.writer.get_engine_version()
    }

    fn get_import(&self, index: PackageIndex) -> Option<&Import> {
        self.writer.get_import(index)
    }

    fn get_export_class_type(&self, index: PackageIndex) -> Option<FName> {
        self.writer.get_export_class_type(index)
    }
}

impl<'name_map, 'writer, Writer: AssetWriter> AssetWriter
    for NameTableWriter<'name_map, 'writer, Writer>
{
    fn write_property_guid(
        &mut self,
        guid: &Option<crate::unreal_types::Guid>,
    ) -> Result<(), crate::error::Error> {
        self.writer.write_property_guid(guid)
    }

    fn write_fname(&mut self, fname: &FName) -> Result<(), crate::error::Error> {
        let mut hasher = DefaultHasher::new();
        fname.content.hash(&mut hasher);

        let hash = hasher.finish();
        let index = self
            .name_map_lookup
            .get(&hash)
            .ok_or_else(|| Error::no_data(format!("No name reference for {}", fname.content)))?;

        self.writer.write_i32::<LittleEndian>(*index)?;
        self.writer.write_i32::<LittleEndian>(fname.index)?;

        Ok(())
    }

    fn write_u8(&mut self, value: u8) -> Result<(), io::Error> {
        self.writer.write_u8(value)
    }

    fn write_i8(&mut self, value: i8) -> Result<(), io::Error> {
        self.writer.write_i8(value)
    }

    fn write_u16<T: byteorder::ByteOrder>(&mut self, value: u16) -> Result<(), io::Error> {
        self.writer.write_u16::<T>(value)
    }

    fn write_i16<T: byteorder::ByteOrder>(&mut self, value: i16) -> Result<(), io::Error> {
        self.writer.write_i16::<T>(value)
    }

    fn write_u32<T: byteorder::ByteOrder>(&mut self, value: u32) -> Result<(), io::Error> {
        self.writer.write_u32::<T>(value)
    }

    fn write_i32<T: byteorder::ByteOrder>(&mut self, value: i32) -> Result<(), io::Error> {
        self.writer.write_i32::<T>(value)
    }

    fn write_u64<T: byteorder::ByteOrder>(&mut self, value: u64) -> Result<(), io::Error> {
        self.writer.write_u64::<T>(value)
    }

    fn write_i64<T: byteorder::ByteOrder>(&mut self, value: i64) -> Result<(), io::Error> {
        self.writer.write_i64::<T>(value)
    }

    fn write_f32<T: byteorder::ByteOrder>(&mut self, value: f32) -> Result<(), io::Error> {
        self.writer.write_f32::<T>(value)
    }

    fn write_f64<T: byteorder::ByteOrder>(&mut self, value: f64) -> Result<(), io::Error> {
        self.writer.write_f64::<T>(value)
    }

    fn write_string(&mut self, value: &Option<String>) -> Result<usize, crate::error::Error> {
        self.writer.write_string(value)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        self.writer.write_all(buf)
    }

    fn write_bool(&mut self, value: bool) -> Result<(), crate::error::Error> {
        self.writer.write_bool(value)
    }
}
