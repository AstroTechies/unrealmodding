//! Usmap file writer

use crate::{
    asset::name_map::NameMap,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    error::Error,
    exports::class_export::ClassExport,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::{
        archive_trait::{ArchiveTrait, ArchiveType},
        archive_writer::{ArchiveWriter, PassthroughArchiveWriter},
    },
    types::PackageIndex,
    Import,
};

use super::Usmap;

/// Usmap file writer
pub struct UsmapWriter<'parent_writer, 'asset, W: ArchiveWriter> {
    /// Parent writer
    parent_writer: &'parent_writer mut W,
    /// Name map
    name_map: &'asset [String],
    /// Custom versions
    custom_versions: &'asset [CustomVersion],
}

impl<'parent_writer, 'asset, W: ArchiveWriter> UsmapWriter<'parent_writer, 'asset, W> {
    /// Write a name to this archive
    pub fn write_name(&mut self, name: &str) -> Result<usize, Error> {
        todo!()
    }
}

impl<'parent_writer, 'asset, W: ArchiveWriter> ArchiveTrait
    for UsmapWriter<'parent_writer, 'asset, W>
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
        self.parent_writer.position()
    }

    fn seek(&mut self, style: std::io::SeekFrom) -> std::io::Result<u64> {
        self.parent_writer.seek(style)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.parent_writer.get_name_map()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        self.parent_writer.get_array_struct_type_override()
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        self.parent_writer.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        self.parent_writer.get_map_value_override()
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.parent_writer.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.parent_writer.get_object_version()
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.parent_writer.get_object_version_ue5()
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        None
    }

    fn get_class_export(&self) -> Option<&ClassExport> {
        self.parent_writer.get_class_export()
    }

    fn get_import(&self, index: PackageIndex) -> Option<Import> {
        self.parent_writer.get_import(index)
    }
}

impl<'parent_writer, 'asset, W: ArchiveWriter> PassthroughArchiveWriter
    for UsmapWriter<'parent_writer, 'asset, W>
{
    type Passthrough = W;

    fn get_passthrough(&mut self) -> &mut Self::Passthrough {
        &mut self.parent_writer
    }
}
