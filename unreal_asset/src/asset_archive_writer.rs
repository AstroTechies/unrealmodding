//! Archive that can be used to write an asset

use std::io::{Seek, Write};

use unreal_asset_base::{
    cast,
    containers::{IndexedMap, NameMap, SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    object_version::{ObjectVersion, ObjectVersionUE5},
    passthrough_archive_writer,
    reader::{ArchiveTrait, ArchiveType, ArchiveWriter},
    types::{FName, PackageIndex, PackageIndexTrait},
    unversioned::Usmap,
    Error, Import,
};
use unreal_asset_exports::Export;

use crate::asset_data::AssetData;

/// Archive that can be used to write UAsset data
pub struct AssetArchiveWriter<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>> {
    /// Parent writer for this writer
    writer: &'parent_writer mut ParentWriter,
    /// Asset data
    asset_data: &'asset AssetData<PackageIndex>,
    /// Asset imports
    imports: &'asset [Import],
    /// Asset name map
    name_map: SharedResource<NameMap>,
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>>
    AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    /// Create a new `AssetArchiveWriter` instance
    pub fn new(
        parent_writer: &'parent_writer mut ParentWriter,
        asset_data: &'asset AssetData<PackageIndex>,
        imports: &'asset [Import],
        name_map: SharedResource<NameMap>,
    ) -> Self {
        AssetArchiveWriter {
            writer: parent_writer,
            asset_data,
            imports,
            name_map,
        }
    }

    /// Get an [`Import`] from this `AssetArchiveWriter`
    pub fn get_import(&self, index: PackageIndex) -> Option<Import> {
        if !index.is_import() {
            return None;
        }

        let index = -index.index - 1;
        if index < 0 || index > self.imports.len() as i32 {
            return None;
        }

        Some(self.imports[index as usize].clone())
    }
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>> ArchiveTrait<PackageIndex>
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    #[inline(always)]
    fn get_archive_type(&self) -> ArchiveType {
        ArchiveType::UAsset
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.asset_data.get_custom_version::<T>()
    }

    fn has_unversioned_properties(&self) -> bool {
        self.asset_data.has_unversioned_properties()
    }

    fn use_event_driven_loader(&self) -> bool {
        self.asset_data.use_event_driven_loader
    }

    fn position(&mut self) -> u64 {
        self.writer.position()
    }

    fn set_position(&mut self, pos: u64) -> std::io::Result<()> {
        self.writer.set_position(pos)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.array_struct_type_override
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.map_key_override
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.map_value_override
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.asset_data.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.asset_data.object_version
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.asset_data.object_version_ue5
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        self.asset_data.mappings.as_ref()
    }

    fn get_parent_class_export_name(&self) -> Option<FName> {
        self.asset_data
            .exports
            .iter()
            .find_map(|e| cast!(Export, ClassExport, e))
            .and_then(|e| self.get_import(e.struct_export.super_struct))
            .and_then(|e| self.get_import(e.outer_index))
            .map(|e| e.object_name)
    }

    fn get_object_name(&self, index: PackageIndex) -> Option<FName> {
        self.get_object_name_packageindex(index)
    }

    fn get_object_name_packageindex(&self, index: PackageIndex) -> Option<FName> {
        self.get_import(index).map(|e| e.object_name)
    }
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>> ArchiveWriter<PackageIndex>
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    passthrough_archive_writer!(writer);
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>> Write
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter<PackageIndex>> Seek
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.writer.seek(pos)
    }
}
