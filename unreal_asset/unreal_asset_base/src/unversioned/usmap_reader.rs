//! Usmap file reader

use std::io::{Read, Seek};

use byteorder::{ReadBytesExt, LE};

use crate::{
    containers::name_map::NameMap,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    error::{Error, UsmapError},
    object_version::{ObjectVersion, ObjectVersionUE5},
    passthrough_archive_reader,
    reader::{
        archive_reader::ArchiveReader,
        archive_trait::{ArchiveTrait, ArchiveType},
    },
    types::{FName, PackageIndex},
};

use super::Usmap;

/// Usmap file reader
pub struct UsmapReader<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>> {
    /// Parent reader
    parent_reader: &'parent_reader mut R,
    /// Name map
    name_map: &'asset [String],
    /// Custom versions
    custom_versions: &'asset [CustomVersion],
}

impl<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>>
    UsmapReader<'parent_reader, 'asset, R>
{
    /// Create a new `UsmapReader` instance
    pub fn new(
        parent_reader: &'parent_reader mut R,
        name_map: &'asset [String],
        custom_versions: &'asset [CustomVersion],
    ) -> Self {
        UsmapReader {
            parent_reader,
            name_map,
            custom_versions,
        }
    }

    /// Read a name from this archive
    pub fn read_name(&mut self) -> Result<String, Error> {
        let index = self.read_i32::<LE>()?;
        if index < 0 {
            return Err(UsmapError::name_map_index_out_of_range(self.name_map.len(), index).into());
        }
        self.name_map.get(index as usize).cloned().ok_or_else(|| {
            UsmapError::name_map_index_out_of_range(self.name_map.len(), index).into()
        })
    }
}

impl<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>> ArchiveTrait<PackageIndex>
    for UsmapReader<'parent_reader, 'asset, R>
{
    fn get_archive_type(&self) -> ArchiveType {
        ArchiveType::Usmap
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.custom_versions
            .iter()
            .find(|e| e.guid == T::GUID)
            .cloned()
            .unwrap_or_else(|| CustomVersion::new(T::GUID, 0))
    }

    fn has_unversioned_properties(&self) -> bool {
        false
    }

    fn use_event_driven_loader(&self) -> bool {
        false
    }

    fn position(&mut self) -> u64 {
        self.parent_reader.position()
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.parent_reader.get_name_map()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        self.parent_reader.get_array_struct_type_override()
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        self.parent_reader.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        self.parent_reader.get_map_value_override()
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.parent_reader.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.parent_reader.get_object_version()
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.parent_reader.get_object_version_ue5()
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        None
    }

    fn get_parent_class_export_name(&self) -> Option<FName> {
        self.parent_reader.get_parent_class_export_name()
    }

    fn get_object_name(&mut self, index: PackageIndex) -> Option<FName> {
        self.parent_reader.get_object_name(index)
    }

    fn get_object_name_packageindex(&self, index: PackageIndex) -> Option<FName> {
        self.parent_reader.get_object_name_packageindex(index)
    }
}

impl<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>> ArchiveReader<PackageIndex>
    for UsmapReader<'parent_reader, 'asset, R>
{
    passthrough_archive_reader!(parent_reader);
}

impl<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>> Read
    for UsmapReader<'parent_reader, 'asset, R>
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.parent_reader.read(buf)
    }
}

impl<'parent_reader, 'asset, R: ArchiveReader<PackageIndex>> Seek
    for UsmapReader<'parent_reader, 'asset, R>
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.parent_reader.seek(pos)
    }
}
