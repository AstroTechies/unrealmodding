//! Archive that can be used to write an asset

use crate::{
    asset::{name_map::NameMap, AssetData},
    cast,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    exports::{class_export::ClassExport, Export},
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    types::PackageIndex,
    unversioned::Usmap,
    Import,
};

use super::{
    archive_trait::ArchiveTrait,
    archive_writer::{ArchiveWriter, PassthroughArchiveWriter},
};

/// Archive that can be used to write an asset
pub struct AssetArchiveWriter<'parent_writer, 'asset, ParentWriter: ArchiveWriter> {
    /// Parent writer for this writer
    writer: &'parent_writer mut ParentWriter,
    /// Asset data
    asset_data: &'asset AssetData,
    /// Asset imports
    imports: &'asset [Import],
    /// Asset name map
    name_map: SharedResource<NameMap>,
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter>
    AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    /// Create a new `AssetArchiveWriter` instance
    pub fn new(
        parent_writer: &'parent_writer mut ParentWriter,
        asset_data: &'asset AssetData,
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
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter> ArchiveTrait
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.asset_data.get_custom_version::<T>()
    }

    fn has_unversioned_properties(&self) -> bool {
        self.asset_data
            .package_flags
            .contains(EPackageFlags::PKG_UNVERSIONED_PROPERTIES)
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

    fn seek(&mut self, style: std::io::SeekFrom) -> std::io::Result<u64> {
        self.writer.seek(style)
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

    fn get_class_export(&self) -> Option<&ClassExport> {
        self.asset_data
            .exports
            .iter()
            .find_map(|e| cast!(Export, ClassExport, e))
    }

    fn get_import(&self, index: PackageIndex) -> Option<&Import> {
        if !index.is_import() {
            return None;
        }

        let index = -index.index - 1;
        if index < 0 || index > self.imports.len() as i32 {
            return None;
        }

        Some(&self.imports[index as usize])
    }
}

impl<'parent_writer, 'asset, ParentWriter: ArchiveWriter> PassthroughArchiveWriter
    for AssetArchiveWriter<'parent_writer, 'asset, ParentWriter>
{
    type Passthrough = ParentWriter;

    #[inline(always)]
    fn get_passthrough(&mut self) -> &mut Self::Passthrough {
        self.writer
    }
}
