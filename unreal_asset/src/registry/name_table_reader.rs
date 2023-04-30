//! Asset registry NameTableReader
use std::io::{self, SeekFrom};

use byteorder::LittleEndian;

use crate::asset::name_map::NameMap;
use crate::containers::indexed_map::IndexedMap;
use crate::containers::shared_resource::SharedResource;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::EngineVersion;
use crate::error::Error;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{archive_reader::ArchiveReader, archive_trait::ArchiveTrait};
use crate::types::{FName, Guid, PackageIndex};
use crate::Import;

/// Used for reading NameTable entries by modifying the behavior
/// of some of the value read methods.
pub struct NameTableReader<'reader, Reader: ArchiveReader> {
    /// Reader
    reader: &'reader mut Reader,
    /// Name map
    pub(crate) name_map: SharedResource<NameMap>,
}

impl<'reader, Reader: ArchiveReader> NameTableReader<'reader, Reader> {
    /// Create a new `NameTableReader` from another `Reader`
    pub(crate) fn new(reader: &'reader mut Reader) -> Result<Self, Error> {
        let name_offset = reader.read_i64::<LittleEndian>()?;
        // todo: length checking

        let mut name_map = NameMap::new();
        if name_offset > 0 {
            let original_offset = reader.position();
            reader.seek(SeekFrom::Start(name_offset as u64))?;

            let name_count = reader.read_i32::<LittleEndian>()?;
            if name_count < 0 {
                return Err(Error::invalid_file("Corrupted file".to_string()));
            }

            for i in 0..name_count {
                let name = reader.read_fstring()?.ok_or_else(|| {
                    Error::invalid_file(format!("Name table entry {i} is missing a name"))
                })?;
                name_map.get_mut().add_name_reference(name, false);

                if reader.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
                    let _non_case_preserving_hash = reader.read_u16::<LittleEndian>()?;
                    let _case_preserving_hash = reader.read_u16::<LittleEndian>()?;
                }
            }

            reader.seek(SeekFrom::Start(original_offset))?;
        }
        Ok(NameTableReader { reader, name_map })
    }
}

impl<'reader, Reader: ArchiveReader> ArchiveTrait for NameTableReader<'reader, Reader> {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.reader.get_custom_version::<T>()
    }

    fn position(&mut self) -> u64 {
        self.reader.position()
    }

    fn set_position(&mut self, pos: u64) {
        self.reader.set_position(pos)
    }

    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        self.reader.seek(style)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn get_name_reference(&self, index: i32) -> String {
        self.name_map.get_ref().get_name_reference(index)
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_array_struct_type_override()
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_map_value_override()
    }

    fn get_parent_class(&self) -> Option<crate::ParentClassInfo> {
        self.reader.get_parent_class()
    }

    fn get_parent_class_cached(&mut self) -> Option<&crate::ParentClassInfo> {
        self.reader.get_parent_class_cached()
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.reader.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.reader.get_object_version()
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.reader.get_object_version_ue5()
    }

    fn get_import(&self, index: PackageIndex) -> Option<&Import> {
        self.reader.get_import(index)
    }

    fn get_export_class_type(&self, index: PackageIndex) -> Option<FName> {
        self.reader.get_export_class_type(index)
    }

    fn add_fname(&mut self, value: &str) -> FName {
        self.reader.add_fname(value)
    }

    fn add_fname_with_number(&mut self, value: &str, number: i32) -> FName {
        self.reader.add_fname_with_number(value, number)
    }

    fn get_mappings(&self) -> Option<&crate::unversioned::Usmap> {
        self.reader.get_mappings()
    }

    fn has_unversioned_properties(&self) -> bool {
        self.reader.has_unversioned_properties()
    }
}

impl<'reader, Reader: ArchiveReader> ArchiveReader for NameTableReader<'reader, Reader> {
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        self.reader.read_property_guid()
    }

    fn read_fname(&mut self) -> Result<FName, Error> {
        let index = self.reader.read_i32::<LittleEndian>()?;
        let number = self.reader.read_i32::<LittleEndian>()?;
        Ok(self.name_map.get_ref().create_fname(index, number))
    }

    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let mut array = Vec::with_capacity(length as usize);
        for _ in 0..length {
            array.push(getter(self)?);
        }
        Ok(array)
    }

    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.reader.read_i32::<LittleEndian>()?;
        self.read_array_with_length(length, getter)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        self.reader.read_u8()
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        self.reader.read_i8()
    }

    fn read_u16<T: byteorder::ByteOrder>(&mut self) -> io::Result<u16> {
        self.reader.read_u16::<T>()
    }

    fn read_i16<T: byteorder::ByteOrder>(&mut self) -> io::Result<i16> {
        self.reader.read_i16::<T>()
    }

    fn read_u32<T: byteorder::ByteOrder>(&mut self) -> io::Result<u32> {
        self.reader.read_u32::<T>()
    }

    fn read_i32<T: byteorder::ByteOrder>(&mut self) -> io::Result<i32> {
        self.reader.read_i32::<T>()
    }

    fn read_u64<T: byteorder::ByteOrder>(&mut self) -> io::Result<u64> {
        self.reader.read_u64::<T>()
    }

    fn read_i64<T: byteorder::ByteOrder>(&mut self) -> io::Result<i64> {
        self.reader.read_i64::<T>()
    }

    fn read_f32<T: byteorder::ByteOrder>(&mut self) -> io::Result<f32> {
        self.reader.read_f32::<T>()
    }

    fn read_f64<T: byteorder::ByteOrder>(&mut self) -> io::Result<f64> {
        self.reader.read_f64::<T>()
    }

    fn read_fstring(&mut self) -> Result<Option<String>, Error> {
        self.reader.read_fstring()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        self.reader.read_bool()
    }
}
