//! Binary archive writer

use std::io::{self, Seek, Write};
use std::marker::PhantomData;

use unreal_helpers::{Guid, UnrealWriteExt};

use crate::containers::{IndexedMap, NameMap, SharedResource};
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{
    archive_trait::{ArchiveTrait, ArchiveType},
    ArchiveWriter,
};
use crate::types::{FName, PackageIndex, PackageIndexTrait};
use crate::unversioned::Usmap;
use crate::Error;

/// A binary writer
pub struct RawWriter<'cursor, Index: PackageIndexTrait, W: Write + Seek> {
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
    /// Marker
    _marker: PhantomData<Index>,
}

impl<'cursor, Index: PackageIndexTrait, W: Write + Seek> RawWriter<'cursor, Index, W> {
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
            _marker: PhantomData,
        }
    }
}

impl<'cursor, Index: PackageIndexTrait, W: Write + Seek> ArchiveTrait<Index>
    for RawWriter<'cursor, Index, W>
{
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

    fn get_parent_class_export_name(&self) -> Option<FName> {
        None
    }

    fn get_object_name(&mut self, _: Index) -> Option<FName> {
        None
    }

    fn get_object_name_packageindex(&self, _: PackageIndex) -> Option<FName> {
        None
    }
}

impl<'cursor, Index: PackageIndexTrait, W: Write + Seek> ArchiveWriter<Index>
    for RawWriter<'cursor, Index, W>
{
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error> {
        Ok(self.cursor.write_fstring(value)?)
    }

    fn write_guid(&mut self, guid: &Guid) -> io::Result<()> {
        self.cursor.write_guid(guid)
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.cursor.write_bool(value)
    }
}

impl<'cursor, Index: PackageIndexTrait, W: Write + Seek> Write for RawWriter<'cursor, Index, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.cursor.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.cursor.flush()
    }
}

impl<'cursor, Index: PackageIndexTrait, W: Write + Seek> Seek for RawWriter<'cursor, Index, W> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(pos)
    }
}
