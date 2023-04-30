//! Binary archive reader

use std::io::{self, Cursor, Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use unreal_helpers::UnrealReadExt;

use crate::asset::name_map::NameMap;
use crate::containers::indexed_map::IndexedMap;
use crate::containers::shared_resource::SharedResource;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::error::Error;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{archive_reader::ArchiveReader, archive_trait::ArchiveTrait};
use crate::types::{FName, Guid, PackageIndex};
use crate::Import;

/// A binary reader
pub struct RawReader {
    /// Reader cursor
    cursor: Cursor<Vec<u8>>,
    /// Object version
    object_version: ObjectVersion,
    /// UE5 object version
    object_version_ue5: ObjectVersionUE5,

    /// Dummy name map
    dummy_name_map: SharedResource<NameMap>,
    /// Empty map
    empty_map: IndexedMap<String, String>,
}

impl RawReader {
    /// Create a new instance of `RawReader` with the specified object versions
    pub fn new(
        cursor: Cursor<Vec<u8>>,
        object_version: ObjectVersion,
        object_version_ue5: ObjectVersionUE5,
    ) -> Self {
        RawReader {
            cursor,
            object_version,
            object_version_ue5,
            dummy_name_map: NameMap::new(),
            empty_map: IndexedMap::new(),
        }
    }
}

impl ArchiveTrait for RawReader {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        CustomVersion::new([0u8; 16], 0)
    }

    fn position(&mut self) -> u64 {
        self.cursor.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos)
    }

    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(style)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.dummy_name_map.clone()
    }

    fn get_name_reference(&self, _: i32) -> String {
        "".to_string()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        &self.empty_map
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        &self.empty_map
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        &self.empty_map
    }

    fn get_parent_class(&self) -> Option<crate::ParentClassInfo> {
        None
    }

    fn get_parent_class_cached(&mut self) -> Option<&crate::ParentClassInfo> {
        None
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
        FName::new_dummy(value.to_string(), 0)
    }

    fn add_fname_with_number(&mut self, value: &str, number: i32) -> FName {
        FName::new_dummy(value.to_string(), number)
    }

    fn get_mappings(&self) -> Option<&crate::unversioned::Usmap> {
        None
    }

    fn has_unversioned_properties(&self) -> bool {
        false
    }
}

impl ArchiveReader for RawReader {
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        Ok(None)
    }

    fn read_fname(&mut self) -> Result<FName, Error> {
        let string = self.read_fstring()?.unwrap_or_else(|| "None".to_string());
        Ok(FName::new_dummy(string, 0))
    }

    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let mut result = Vec::new();
        for _ in 0..length {
            result.push(getter(self)?);
        }
        Ok(result)
    }

    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.read_i32::<LittleEndian>()?;
        self.read_array_with_length(length, getter)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor.read_i8()
    }

    fn read_u16<T: byteorder::ByteOrder>(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<T>()
    }

    fn read_i16<T: byteorder::ByteOrder>(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<T>()
    }

    fn read_u32<T: byteorder::ByteOrder>(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<T>()
    }

    fn read_i32<T: byteorder::ByteOrder>(&mut self) -> io::Result<i32> {
        self.cursor.read_i32::<T>()
    }

    fn read_u64<T: byteorder::ByteOrder>(&mut self) -> io::Result<u64> {
        self.cursor.read_u64::<T>()
    }

    fn read_i64<T: byteorder::ByteOrder>(&mut self) -> io::Result<i64> {
        self.cursor.read_i64::<T>()
    }

    fn read_f32<T: byteorder::ByteOrder>(&mut self) -> io::Result<f32> {
        self.cursor.read_f32::<T>()
    }

    fn read_f64<T: byteorder::ByteOrder>(&mut self) -> io::Result<f64> {
        self.cursor.read_f64::<T>()
    }

    fn read_fstring(&mut self) -> Result<Option<String>, Error> {
        Ok(self.cursor.read_fstring()?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.cursor.read_exact(buf)
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }
}
