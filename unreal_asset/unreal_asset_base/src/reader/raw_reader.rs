//! Binary archive reader

use std::io::{self, Read, Seek};

use byteorder::ReadBytesExt;

use unreal_helpers::{read_ext::read_fstring_len, Guid, UnrealReadExt};

use crate::containers::chain::Chain;
use crate::containers::indexed_map::IndexedMap;
use crate::containers::name_map::NameMap;
use crate::containers::shared_resource::SharedResource;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::{guess_engine_version, EngineVersion};
use crate::error::Error;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::{archive_reader::ArchiveReader, archive_trait::ArchiveTrait};
use crate::types::fname::FName;
use crate::types::{PackageIndex, SerializedNameHeader};
use crate::unversioned::Usmap;
use crate::Import;

use super::archive_trait::ArchiveType;

/// A binary reader
pub struct RawReader<C: Read + Seek> {
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
}

impl<C: Read + Seek> RawReader<C> {
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
        }
    }
}

impl<C: Read + Seek> ArchiveTrait for RawReader<C> {
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

    fn get_parent_class_export_name(&self) -> Option<FName> {
        None
    }

    fn get_import(&self, _: PackageIndex) -> Option<Import> {
        None
    }
}

impl<C: Read + Seek> ArchiveReader for RawReader<C> {
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

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.cursor.read_exact(buf)
    }

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

    fn read_guid(&mut self) -> io::Result<Guid> {
        self.cursor.read_guid()
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        self.cursor.read_bool()
    }
}
