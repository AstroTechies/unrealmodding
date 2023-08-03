//! Binary archive writer

use std::io::{self, Seek, Write};

use byteorder::WriteBytesExt;

use unreal_helpers::{Guid, UnrealWriteExt};

use crate::asset::name_map::NameMap;
use crate::containers::indexed_map::IndexedMap;
use crate::containers::shared_resource::SharedResource;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::error::Error;
use crate::exports::class_export::ClassExport;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{archive_trait::ArchiveTrait, archive_writer::ArchiveWriter};
use crate::types::PackageIndex;
use crate::unversioned::Usmap;
use crate::Import;

use super::archive_trait::ArchiveType;

/// A binary writer
pub struct RawWriter<'cursor, W: Write + Seek> {
    /// Writer cursor
    cursor: &'cursor mut W,
    /// Object version
    object_version: ObjectVersion,
    /// UE5 object version
    object_version_ue5: ObjectVersionUE5,
    /// Does the reader use the event driven loader
    use_event_driven_loader: bool,
    /// Name map
    name_map: SharedResource<NameMap>,
    /// Empty map
    empty_map: IndexedMap<String, String>,
}

impl<'cursor, W: Write + Seek> RawWriter<'cursor, W> {
    /// Create a new instance of `RawWriter` with the specified object versions
    pub fn new(
        cursor: &'cursor mut W,
        object_version: ObjectVersion,
        object_version_ue5: ObjectVersionUE5,
        use_event_driven_loader: bool,
        name_map: SharedResource<NameMap>,
    ) -> Self {
        RawWriter {
            cursor,
            object_version,
            object_version_ue5,
            use_event_driven_loader,
            name_map,
            empty_map: IndexedMap::new(),
        }
    }
}

impl<'cursor, W: Write + Seek> ArchiveTrait for RawWriter<'cursor, W> {
    #[inline(always)]
    fn get_archive_type(&self) -> ArchiveType {
        ArchiveType::Raw
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        CustomVersion::new(Guid::default(), 0)
    }

    fn has_unversioned_properties(&self) -> bool {
        false
    }

    fn use_event_driven_loader(&self) -> bool {
        self.use_event_driven_loader
    }

    fn position(&mut self) -> u64 {
        self.cursor.stream_position().unwrap_or_default()
    }

    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(style)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
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

    fn get_engine_version(&self) -> EngineVersion {
        guess_engine_version(self.object_version, self.object_version_ue5, &[])
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.object_version
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.object_version_ue5
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        None
    }

    fn get_class_export(&self) -> Option<&ClassExport> {
        None
    }

    fn get_import(&self, _: PackageIndex) -> Option<Import> {
        None
    }
}

impl<'cursor, W: Write + Seek> ArchiveWriter for RawWriter<'cursor, W> {
    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.cursor.write_u8(value)
    }

    fn write_i8(&mut self, value: i8) -> io::Result<()> {
        self.cursor.write_i8(value)
    }

    fn write_u16<T: byteorder::ByteOrder>(&mut self, value: u16) -> io::Result<()> {
        self.cursor.write_u16::<T>(value)
    }

    fn write_i16<T: byteorder::ByteOrder>(&mut self, value: i16) -> io::Result<()> {
        self.cursor.write_i16::<T>(value)
    }

    fn write_u32<T: byteorder::ByteOrder>(&mut self, value: u32) -> io::Result<()> {
        self.cursor.write_u32::<T>(value)
    }

    fn write_i32<T: byteorder::ByteOrder>(&mut self, value: i32) -> io::Result<()> {
        self.cursor.write_i32::<T>(value)
    }

    fn write_u64<T: byteorder::ByteOrder>(&mut self, value: u64) -> io::Result<()> {
        self.cursor.write_u64::<T>(value)
    }

    fn write_i64<T: byteorder::ByteOrder>(&mut self, value: i64) -> io::Result<()> {
        self.cursor.write_i64::<T>(value)
    }

    fn write_f32<T: byteorder::ByteOrder>(&mut self, value: f32) -> io::Result<()> {
        self.cursor.write_f32::<T>(value)
    }

    fn write_f64<T: byteorder::ByteOrder>(&mut self, value: f64) -> io::Result<()> {
        self.cursor.write_f64::<T>(value)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.cursor.write_all(buf)
    }

    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error> {
        Ok(self.cursor.write_fstring(value)?)
    }

    fn write_guid(&mut self, guid: crate::Guid) -> io::Result<()> {
        self.cursor.write_guid(guid)
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.cursor.write_bool(value)
    }
}
