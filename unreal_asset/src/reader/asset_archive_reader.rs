//! Archive that can be used to read an asset

use crate::{
    asset::{name_map::NameMap, AssetData},
    cast,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    exports::{class_export::ClassExport, Export},
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    types::{fname::FName, PackageIndex},
    unversioned::Usmap,
    Import,
};

use super::{
    archive_reader::{ArchiveReader, PassthroughArchiveReader},
    archive_trait::ArchiveTrait,
};

/// Archive that can be used to read an asset
pub struct AssetArchiveReader<'parent_reader, 'asset, ParentReader: ArchiveReader> {
    /// Parent reader for this reader
    reader: &'parent_reader mut ParentReader,
    /// Asset data
    asset_data: &'asset AssetData,
    /// Asset imports
    imports: &'asset [Import],
    /// Asset name map
    name_map: SharedResource<NameMap>,
}

impl<'parent_reader, 'asset, ParentReader: ArchiveReader>
    AssetArchiveReader<'parent_reader, 'asset, ParentReader>
{
    /// Create a new `AssetArchiveReader` instance
    pub fn new(
        parent_reader: &'parent_reader mut ParentReader,
        asset_data: &'asset AssetData,
        imports: &'asset [Import],
        name_map: SharedResource<NameMap>,
    ) -> Self {
        AssetArchiveReader {
            reader: parent_reader,
            asset_data,
            imports,
            name_map,
        }
    }
}

impl<'parent_reader, 'asset, ParentReader: ArchiveReader> ArchiveTrait
    for AssetArchiveReader<'parent_reader, 'asset, ParentReader>
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
        self.reader.position()
    }

    fn set_position(&mut self, pos: u64) -> std::io::Result<()> {
        self.reader.set_position(pos)
    }

    fn seek(&mut self, style: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(style)
    }

    fn add_fname(&mut self, value: &str) -> FName {
        self.name_map.get_mut().add_fname(value)
    }

    fn add_fname_with_number(&mut self, value: &str, number: i32) -> crate::types::fname::FName {
        self.name_map.get_mut().add_fname_with_number(value, number)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn get_name_reference(&self, index: i32) -> String {
        self.name_map.get_ref().get_name_reference(index)
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

    /// Searches for and returns this asset's CLassExport, if one exists
    fn get_class_export(&self) -> Option<&ClassExport> {
        self.asset_data
            .exports
            .iter()
            .find_map(|e| cast!(Export, ClassExport, e))
    }
}

impl<'parent_reader, 'asset, ParentReader: ArchiveReader> PassthroughArchiveReader
    for AssetArchiveReader<'parent_reader, 'asset, ParentReader>
{
    type Passthrough = ParentReader;

    #[inline(always)]
    fn get_passthrough(&mut self) -> &mut Self::Passthrough {
        self.reader
    }
}
