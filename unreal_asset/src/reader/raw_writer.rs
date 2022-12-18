use std::collections::HashMap;
use std::io::{self, Cursor, Seek, Write};

use byteorder::WriteBytesExt;

use crate::cursor_ext::CursorExt;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{asset_trait::AssetTrait, asset_writer::AssetWriter};
use crate::unreal_types::{FName, PackageIndex};
use crate::Import;

/// A binary writer
pub struct RawWriter<'cursor> {
    cursor: &'cursor mut Cursor<Vec<u8>>,
    object_version: ObjectVersion,
    object_version_ue5: ObjectVersionUE5,

    empty_map: HashMap<String, String>,
}

impl<'cursor> RawWriter<'cursor> {
    pub fn new(
        cursor: &'cursor mut Cursor<Vec<u8>>,
        object_version: ObjectVersion,
        object_version_ue5: ObjectVersionUE5,
    ) -> Self {
        RawWriter {
            cursor,
            object_version,
            object_version_ue5,
            empty_map: HashMap::new(),
        }
    }
}

impl<'cursor> AssetTrait for RawWriter<'cursor> {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        CustomVersion::new([0u8; 16], 0)
    }

    fn position(&self) -> u64 {
        self.cursor.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos)
    }

    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(style)
    }

    fn get_name_map_index_list(&self) -> &[String] {
        &[]
    }

    fn get_name_reference(&self, _: i32) -> String {
        "".to_string()
    }

    fn get_map_key_override(&self) -> &HashMap<String, String> {
        &self.empty_map
    }

    fn get_map_value_override(&self) -> &HashMap<String, String> {
        &self.empty_map
    }

    fn get_engine_version(&self) -> EngineVersion {
        guess_engine_version(self.object_version, self.object_version_ue5, &[])
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.object_version
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.object_version_ue5
    }

    fn get_import(&self, _index: PackageIndex) -> Option<&Import> {
        None
    }

    fn get_export_class_type(&self, _index: PackageIndex) -> Option<FName> {
        None
    }

    fn add_fname(&mut self, value: &str) -> FName {
        FName::new(value.to_string(), 0)
    }

    fn add_fname_with_number(&mut self, value: &str, number: i32) -> FName {
        FName::new(value.to_string(), number)
    }

    fn get_mappings(&self) -> Option<&crate::unversioned::Usmap> {
        None
    }
}

impl<'cursor> AssetWriter for RawWriter<'cursor> {
    fn write_property_guid(
        &mut self,
        guid: &Option<crate::unreal_types::Guid>,
    ) -> Result<(), crate::error::Error> {
        if self.object_version >= ObjectVersion::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            self.cursor.write_bool(guid.is_some())?;
            if let Some(ref data) = guid {
                self.cursor.write_all(data)?;
            }
        }
        Ok(())
    }

    fn write_fname(&mut self, fname: &FName) -> Result<(), crate::error::Error> {
        self.cursor.write_string(&Some(fname.content.clone()))?; // todo: ref
        Ok(())
    }

    fn write_u8(&mut self, value: u8) -> Result<(), io::Error> {
        self.cursor.write_u8(value)
    }

    fn write_i8(&mut self, value: i8) -> Result<(), io::Error> {
        self.cursor.write_i8(value)
    }

    fn write_u16<T: byteorder::ByteOrder>(&mut self, value: u16) -> Result<(), io::Error> {
        self.cursor.write_u16::<T>(value)
    }

    fn write_i16<T: byteorder::ByteOrder>(&mut self, value: i16) -> Result<(), io::Error> {
        self.cursor.write_i16::<T>(value)
    }

    fn write_u32<T: byteorder::ByteOrder>(&mut self, value: u32) -> Result<(), io::Error> {
        self.cursor.write_u32::<T>(value)
    }

    fn write_i32<T: byteorder::ByteOrder>(&mut self, value: i32) -> Result<(), io::Error> {
        self.cursor.write_i32::<T>(value)
    }

    fn write_u64<T: byteorder::ByteOrder>(&mut self, value: u64) -> Result<(), io::Error> {
        self.cursor.write_u64::<T>(value)
    }

    fn write_i64<T: byteorder::ByteOrder>(&mut self, value: i64) -> Result<(), io::Error> {
        self.cursor.write_i64::<T>(value)
    }

    fn write_f32<T: byteorder::ByteOrder>(&mut self, value: f32) -> Result<(), io::Error> {
        self.cursor.write_f32::<T>(value)
    }

    fn write_f64<T: byteorder::ByteOrder>(&mut self, value: f64) -> Result<(), io::Error> {
        self.cursor.write_f64::<T>(value)
    }

    fn write_string(&mut self, value: &Option<String>) -> Result<usize, crate::error::Error> {
        self.cursor.write_string(value)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        self.cursor.write_all(buf)
    }

    fn write_bool(&mut self, value: bool) -> Result<(), crate::error::Error> {
        self.cursor.write_bool(value)
    }
}
