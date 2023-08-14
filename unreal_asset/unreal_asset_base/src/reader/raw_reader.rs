//! Binary archive reader

use std::io::{self, Read, Seek};
use std::marker::PhantomData;

use unreal_helpers::read_ext::read_fstring_len_noterm;
use unreal_helpers::{read_ext::read_fstring_len, Guid, UnrealReadExt};

use crate::containers::{Chain, IndexedMap, NameMap, SharedResource};
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{
    archive_trait::{ArchiveTrait, ArchiveType},
    ArchiveReader,
};
use crate::types::{FName, PackageIndex, PackageIndexTrait, SerializedNameHeader};
use crate::unversioned::Usmap;
use crate::Error;


/// A binary reader
pub struct RawReader<Index: PackageIndexTrait, C: Read + Seek> {
    /// Reader cursor
    cursor: Chain<C>,
    /// Object version
    pub object_version: ObjectVersion,
    /// UE5 object version
    pub object_version_ue5: ObjectVersionUE5,
    /// Does the reader use the event driven loader
    pub use_event_driven_loader: bool,
    /// Name map
    pub name_map: SharedResource<NameMap>,
    /// Empty map
    empty_map: IndexedMap<String, String>,

    /// Marker
    _marker: PhantomData<Index>,
}

impl<Index: PackageIndexTrait, C: Read + Seek> RawReader<Index, C> {
    /// Create a new instance of `RawReader` with the specified object versions and a name map
    pub fn new(
        cursor: Chain<C>,
        object_version: ObjectVersion,
        object_version_ue5: ObjectVersionUE5,
        use_event_driven_loader: bool,
        name_map: SharedResource<NameMap>,
    ) -> Self {
        RawReader {
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

impl<Index: PackageIndexTrait, C: Read + Seek> ArchiveTrait<Index> for RawReader<Index, C> {
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

    fn get_object_name(&self, _: Index) -> Option<FName> {
        None
    }

    fn get_object_name_packageindex(&self, _: PackageIndex) -> Option<FName> {
        None
    }
}

impl<Index: PackageIndexTrait, C: Read + Seek> ArchiveReader<Index> for RawReader<Index, C> {
    fn read_fstring(&mut self) -> Result<Option<String>, Error> {
        Ok(self.cursor.read_fstring()?)
    }

    fn read_fstring_name_header(
        &mut self,
        serialized_name_header: SerializedNameHeader,
    ) -> Result<Option<String>, Error> {
        if serialized_name_header.len == 0 {
            return Ok(None);
        }

        Ok(read_fstring_len(
            &mut self.cursor,
            serialized_name_header.len,
            serialized_name_header.is_wide,
        )?)
    }

    fn read_fstring_len_noterm(
        &mut self,
        len: i32,
        is_wide: bool,
    ) -> Result<Option<String>, Error> {
        Ok(read_fstring_len_noterm(&mut self.cursor, len, is_wide)?)
    }

    fn read_guid(&mut self) -> io::Result<Guid> {
        self.cursor.read_guid()
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        self.cursor.read_bool()
    }
}

impl<Index: PackageIndexTrait, C: Read + Seek> Read for RawReader<Index, C> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl<Index: PackageIndexTrait, C: Read + Seek> Seek for RawReader<Index, C> {
    #[inline(always)]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(pos)
    }
}
