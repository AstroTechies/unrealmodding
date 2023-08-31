//! Main [`Asset`] type

use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use byteorder::{ReadBytesExt, WriteBytesExt, BE, LE};

use unreal_asset_base::flags::EObjectFlags;
use unreal_asset_base::passthrough_archive_reader;
use unreal_asset_base::types::PackageIndexTrait;
use unreal_asset_base::{
    cast,
    containers::{Chain, IndexedMap, NameMap, SharedResource},
    crc,
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    enums::ECustomVersionSerializationFormat,
    error::Error,
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::{ArchiveReader, ArchiveTrait, ArchiveType, ArchiveWriter, RawReader, RawWriter},
    types::{fname::FNameContainer, FName, GenerationInfo, PackageIndex},
    unversioned::Usmap,
    FNameContainer, Guid, Import,
};
use unreal_asset_exports::{BaseExport, Export, ExportBaseTrait, ExportNormalTrait, ExportTrait};
use unreal_asset_properties::world_tile_property::FWorldTileInfo;

use crate::asset_archive_writer::AssetArchiveWriter;
use crate::asset_data::{AssetData, AssetTrait, ExportReaderTrait};
use crate::fengineversion::FEngineVersion;
use crate::UE4_ASSET_MAGIC;

/// Parent Class Info
#[derive(FNameContainer, Debug, Clone, Eq, PartialEq)]
pub struct ParentClassInfo {
    /// Parent classpath
    pub parent_class_path: FName,
    /// Parent class export name
    pub parent_class_export_name: FName,
}

/// UAsset export map entry
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UAssetExportMapEntry {
    /// Class index
    pub class_index: PackageIndex,
    /// Super index
    pub super_index: PackageIndex,
    /// Template index
    pub template_index: PackageIndex,
    /// Outer index
    pub outer_index: PackageIndex,
    /// Object name
    pub object_name: FName,
    /// Object flags
    pub object_flags: EObjectFlags,
    /// Serialized size
    pub serial_size: i64,
    /// Serialized offset
    pub serial_offset: i64,
    /// Is forced export
    pub forced_export: bool,
    /// Is not for client
    pub not_for_client: bool,
    /// Is not for server
    pub not_for_server: bool,
    /// Package guid
    pub package_guid: Guid,
    /// Is inherited instance
    pub is_inherited_instance: bool,
    /// Package flags
    pub package_flags: u32,
    /// Is not always loaded for editor game
    pub not_always_loaded_for_editor_game: bool,
    /// Is an asset
    pub is_asset: bool,
    /// Generate public hash
    pub generate_public_hash: bool,
    /// Public export hash. Interpreted as a global import PackageObjectIndex in UE4 assets
    pub public_export_hash: u64,
    /// First dependency serialized offset
    pub first_export_dependency_offset: i32,

    /// Dependencies that should be serialized before this export is serialized
    pub serialization_before_serialization_dependencies: Vec<PackageIndex>,
    /// Dependencies that should be created before this export is serialized
    pub create_before_serialization_dependencies: Vec<PackageIndex>,
    /// Dependencies that should be serialized before this export is created
    pub serialization_before_create_dependencies: Vec<PackageIndex>,
    /// Dependencies that should be created before this export is created
    pub create_before_create_dependencies: Vec<PackageIndex>,
}

impl UAssetExportMapEntry {
    /// Read `UAssetExportMapEntry` from an archive
    pub fn read<R: ArchiveReader<PackageIndex>>(archive: &mut R) -> Result<Self, Error> {
        let mut entry = UAssetExportMapEntry {
            class_index: PackageIndex::new(archive.read_i32::<LE>()?),
            super_index: PackageIndex::new(archive.read_i32::<LE>()?),
            ..Default::default()
        };

        if archive.get_object_version() >= ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS {
            entry.template_index = PackageIndex::new(archive.read_i32::<LE>()?);
        }

        entry.outer_index = PackageIndex::new(archive.read_i32::<LE>()?);
        entry.object_name = archive.read_fname()?;
        entry.object_flags = EObjectFlags::from_bits(archive.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid property flags".to_string()))?;

        if archive.get_object_version() < ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
            entry.serial_size = archive.read_i32::<LE>()? as i64;
            entry.serial_offset = archive.read_i32::<LE>()? as i64;
        } else {
            entry.serial_size = archive.read_i64::<LE>()?;
            entry.serial_offset = archive.read_i64::<LE>()?;
        }

        entry.forced_export = archive.read_i32::<LE>()? == 1;
        entry.not_for_client = archive.read_i32::<LE>()? == 1;
        entry.not_for_server = archive.read_i32::<LE>()? == 1;

        if archive.get_object_version_ue5() < ObjectVersionUE5::REMOVE_OBJECT_EXPORT_PACKAGE_GUID {
            entry.package_guid = archive.read_guid()?;
        }

        if archive.get_object_version_ue5() >= ObjectVersionUE5::TRACK_OBJECT_EXPORT_IS_INHERITED {
            entry.is_inherited_instance = archive.read_i32::<LE>()? == 1;
        }

        entry.package_flags = archive.read_u32::<LE>()?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_LOAD_FOR_EDITOR_GAME {
            entry.not_always_loaded_for_editor_game = archive.read_i32::<LE>()? == 1;
        }

        if archive.get_object_version() >= ObjectVersion::VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
            entry.is_asset = archive.read_i32::<LE>()? == 1;
        }

        if archive.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
            entry.generate_public_hash = archive.read_i32::<LE>()? == 1;
        }

        if archive.get_object_version()
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            entry.first_export_dependency_offset = archive.read_i32::<LE>()?;
            entry.serialization_before_serialization_dependencies =
                Vec::with_capacity(archive.read_i32::<LE>()? as usize);
            entry.create_before_serialization_dependencies =
                Vec::with_capacity(archive.read_i32::<LE>()? as usize);
            entry.serialization_before_create_dependencies =
                Vec::with_capacity(archive.read_i32::<LE>()? as usize);
            entry.create_before_create_dependencies =
                Vec::with_capacity(archive.read_i32::<LE>()? as usize);
        }

        Ok(entry)
    }

    /// Write `UAssetExportMapEntry` to an archive
    pub fn write<W: ArchiveWriter<PackageIndex>>(
        &self,
        archive: &mut W,
        serial_size: i64,
        serial_offset: i64,
        first_export_dependency_offset: i32,
    ) -> Result<(), Error> {
        archive.write_i32::<LE>(self.class_index.index)?;
        archive.write_i32::<LE>(self.super_index.index)?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS {
            archive.write_i32::<LE>(self.template_index.index)?;
        }

        archive.write_i32::<LE>(self.outer_index.index)?;
        archive.write_fname(&self.object_name)?;
        archive.write_u32::<LE>(self.object_flags.bits())?;

        if archive.get_object_version() < ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
            archive.write_i32::<LE>(serial_size as i32)?;
            archive.write_i32::<LE>(serial_offset as i32)?;
        } else {
            archive.write_i64::<LE>(serial_size)?;
            archive.write_i64::<LE>(serial_offset)?;
        }

        archive.write_i32::<LE>(match self.forced_export {
            true => 1,
            false => 0,
        })?;
        archive.write_i32::<LE>(match self.not_for_client {
            true => 1,
            false => 0,
        })?;
        archive.write_i32::<LE>(match self.not_for_server {
            true => 1,
            false => 0,
        })?;

        if archive.get_object_version_ue5() < ObjectVersionUE5::REMOVE_OBJECT_EXPORT_PACKAGE_GUID {
            archive.write_guid(&self.package_guid)?;
        }

        if archive.get_object_version_ue5() >= ObjectVersionUE5::TRACK_OBJECT_EXPORT_IS_INHERITED {
            archive.write_i32::<LE>(match self.is_inherited_instance {
                true => 1,
                false => 0,
            })?;
        }

        archive.write_u32::<LE>(self.package_flags)?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_LOAD_FOR_EDITOR_GAME {
            archive.write_i32::<LE>(match self.not_always_loaded_for_editor_game {
                true => 1,
                false => 0,
            })?;
        }

        if archive.get_object_version() >= ObjectVersion::VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
            archive.write_i32::<LE>(match self.is_asset {
                true => 1,
                false => 0,
            })?;
        }

        if archive.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
            archive.write_i32::<LE>(match self.generate_public_hash {
                true => 1,
                false => 0,
            })?;
        }

        if archive.get_object_version()
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            archive.write_i32::<LE>(first_export_dependency_offset)?;
            archive.write_i32::<LE>(
                self.serialization_before_serialization_dependencies.len() as i32
            )?;
            archive.write_i32::<LE>(self.create_before_serialization_dependencies.len() as i32)?;
            archive.write_i32::<LE>(self.serialization_before_create_dependencies.len() as i32)?;
            archive.write_i32::<LE>(self.create_before_create_dependencies.len() as i32)?;
        }
        Ok(())
    }

    /// Convert `UAssetExportMapEntry` to [`BaseExport`]
    pub fn to_base_export(self) -> BaseExport<PackageIndex> {
        BaseExport {
            class_index: self.class_index,
            super_index: self.super_index,
            template_index: self.template_index,
            outer_index: self.outer_index,
            object_name: self.object_name,
            object_flags: self.object_flags,
            serial_size: self.serial_size,
            serial_offset: self.serial_offset,
            forced_export: self.forced_export,
            not_for_client: self.not_for_client,
            not_for_server: self.not_for_server,
            package_guid: self.package_guid,
            is_inherited_instance: self.is_inherited_instance,
            package_flags: self.package_flags,
            not_always_loaded_for_editor_game: self.not_always_loaded_for_editor_game,
            is_asset: self.is_asset,
            generate_public_hash: self.generate_public_hash,
            public_export_hash: self.public_export_hash,
            first_export_dependency_offset: self.first_export_dependency_offset,
            serialization_before_serialization_dependencies: self
                .serialization_before_serialization_dependencies,
            create_before_serialization_dependencies: self.create_before_serialization_dependencies,
            serialization_before_create_dependencies: self.serialization_before_create_dependencies,
            create_before_create_dependencies: self.create_before_create_dependencies,
        }
    }

    /// Convert [`BaseExport`] to `UAssetExportMapEntry`
    pub fn from_base_export(b: &BaseExport<PackageIndex>) -> Self {
        UAssetExportMapEntry {
            class_index: b.class_index,
            super_index: b.super_index,
            template_index: b.template_index,
            outer_index: b.outer_index,
            object_name: b.object_name.clone(),
            object_flags: b.object_flags,
            serial_size: b.serial_size,
            serial_offset: b.serial_offset,
            forced_export: b.forced_export,
            not_for_client: b.not_for_client,
            not_for_server: b.not_for_server,
            package_guid: b.package_guid,
            is_inherited_instance: b.is_inherited_instance,
            package_flags: b.package_flags,
            not_always_loaded_for_editor_game: b.not_always_loaded_for_editor_game,
            is_asset: b.is_asset,
            generate_public_hash: b.generate_public_hash,
            public_export_hash: b.public_export_hash,
            first_export_dependency_offset: b.first_export_dependency_offset,
            serialization_before_serialization_dependencies: b
                .serialization_before_serialization_dependencies
                .clone(),
            create_before_serialization_dependencies: b
                .create_before_serialization_dependencies
                .clone(),
            serialization_before_create_dependencies: b
                .serialization_before_create_dependencies
                .clone(),
            create_before_create_dependencies: b.create_before_create_dependencies.clone(),
        }
    }
}

/// Asset header
struct AssetHeader {
    /// Name map offset
    name_offset: i32,
    /// Imports offset
    import_offset: i32,
    /// Exports offset
    export_offset: i32,
    /// Dependencies offset
    depends_offset: i32,
    /// Soft package references offset
    soft_package_reference_offset: i32,
    /// Asset registry data offset
    asset_registry_data_offset: i32,
    /// World tile info offset
    world_tile_info_offset: i32,
    /// Preload dependency count
    preload_dependency_count: i32,
    /// Preload dependency offset
    preload_dependency_offset: i32,
    /// Header offset
    header_offset: i32,
    /// Bulk data start offset
    bulk_data_start_offset: i64,
}

//#[derive(Debug)]
/// Unreal Engine uasset
#[derive(FNameContainer)]
pub struct Asset<C: Read + Seek> {
    /// Raw reader
    #[container_ignore]
    pub raw_reader: RawReader<PackageIndex, C>,
    // parsed data
    /// Asset info
    pub info: String,
    /// Asset data
    pub asset_data: AssetData<PackageIndex>,
    /// Legacy file version
    pub legacy_file_version: i32,

    // imports
    // exports
    // depends map
    // soft package reference list
    // asset registry data
    // world tile info
    // preload dependencies
    /// Generations
    #[container_ignore]
    pub generations: Vec<GenerationInfo>,
    /// Asset guid
    pub package_guid: Guid,
    /// Recorded engine version
    #[container_ignore]
    pub engine_version_recorded: FEngineVersion,
    /// Compatible engine version
    #[container_ignore]
    pub engine_version_compatible: FEngineVersion,
    /// Chunk ids
    chunk_ids: Vec<i32>,
    /// Asset source
    pub package_source: u32,
    /// Folder name
    pub folder_name: String,

    // map struct type override
    // override name map hashes
    // todo: isn't this just AssetHeader?
    /// Header offset
    header_offset: i32,
    /// Name count
    name_count: i32,
    /// Name offset
    name_offset: i32,
    /// Names count
    soft_object_paths_count: i32,
    /// Names offset
    soft_object_paths_offset: i32,
    /// Gatherable text data count
    gatherable_text_data_count: i32,
    /// Gatherable text data offset
    gatherable_text_data_offset: i32,
    /// Exports offset
    export_offset: i32,
    /// Imports offset
    import_offset: i32,
    /// Depends offset
    depends_offset: i32,
    /// Soft package reference count
    soft_package_reference_count: i32,
    /// Soft package reference offset
    soft_package_reference_offset: i32,
    /// Searchable names offset
    searchable_names_offset: i32,
    /// Thumbnail table offset
    thumbnail_table_offset: i32,
    /// Compression flags
    compression_flags: u32,
    /// Asset registry data offset
    asset_registry_data_offset: i32,
    /// Bulk data start offset
    pub bulk_data_start_offset: i64,
    /// World tile info offset
    world_tile_info_offset: i32,
    /// Preload dependency count
    preload_dependency_count: i32,
    /// Preload dependency offset
    preload_dependency_offset: i32,
    /// Amount of names referenced from exports
    names_referenced_from_export_data_count: i32,
    /// TOC payload offset
    payload_toc_offset: i64,
    /// Data resource offset
    data_resource_offset: i32,

    /// Overriden name map hashes
    #[container_ignore]
    pub override_name_map_hashes: IndexedMap<String, u32>,
    /// Name map
    #[container_ignore]
    name_map: SharedResource<NameMap>,
    /// Imports
    pub imports: Vec<Import>,
    /// Depends map
    #[container_ignore]
    depends_map: Option<Vec<Vec<i32>>>,
    /// Soft package reference list
    #[container_ignore]
    soft_package_reference_list: Option<Vec<String>>,

    /// Parent class
    parent_class: Option<ParentClassInfo>,
}

impl<'a, C: Read + Seek> Asset<C> {
    /// Create an asset from a binary file
    pub fn new(
        asset_data: C,
        bulk_data: Option<C>,
        engine_version: EngineVersion,
        mappings: Option<Usmap>,
    ) -> Result<Self, Error> {
        let use_event_driven_loader = bulk_data.is_some();

        let chain = Chain::new(asset_data, bulk_data);
        let name_map = NameMap::new();
        let raw_reader = RawReader::new(
            chain,
            ObjectVersion::UNKNOWN,
            ObjectVersionUE5::UNKNOWN,
            use_event_driven_loader,
            name_map.clone(),
        );

        let mut asset = Asset {
            raw_reader,
            info: String::from("Serialized with unrealmodding/uasset"),
            asset_data: AssetData {
                use_event_driven_loader,
                ..Default::default()
            },
            legacy_file_version: 0,
            generations: Vec::new(),
            package_guid: Guid::default(),
            engine_version_recorded: FEngineVersion::unknown(),
            engine_version_compatible: FEngineVersion::unknown(),
            chunk_ids: Vec::new(),
            package_source: 0,
            folder_name: String::from(""),
            header_offset: 0,
            name_count: 0,
            name_offset: 0,
            soft_object_paths_count: 0,
            soft_object_paths_offset: 0,
            gatherable_text_data_count: 0,
            gatherable_text_data_offset: 0,
            export_offset: 0,
            import_offset: 0,
            depends_offset: 0,
            soft_package_reference_count: 0,
            soft_package_reference_offset: 0,
            searchable_names_offset: 0,
            thumbnail_table_offset: 0,
            compression_flags: 0,
            asset_registry_data_offset: 0,
            bulk_data_start_offset: 0,
            world_tile_info_offset: 0,
            preload_dependency_count: 0,
            preload_dependency_offset: 0,
            names_referenced_from_export_data_count: 0,
            payload_toc_offset: 0,
            data_resource_offset: 0,

            override_name_map_hashes: IndexedMap::new(),
            name_map,
            imports: Vec::new(),
            depends_map: None,
            soft_package_reference_list: None,
            parent_class: None,
        };
        asset.set_engine_version(engine_version);
        asset.asset_data.mappings = mappings;
        asset.parse_data()?;
        Ok(asset)
    }

    /// Set asset engine version
    fn set_engine_version(&mut self, engine_version: EngineVersion) {
        self.asset_data.set_engine_version(engine_version);
        self.raw_reader.object_version = self.asset_data.object_version;
        self.raw_reader.object_version_ue5 = self.asset_data.object_version_ue5;
    }

    /// Parse asset header
    fn parse_header(&mut self) -> Result<(), Error> {
        // reuseable buffers for reading

        // seek to start
        self.rewind()?;

        // read and check magic
        if self.read_u32::<BE>()? != UE4_ASSET_MAGIC {
            return Err(Error::invalid_file(
                "File is not a valid uasset file".to_string(),
            ));
        }

        // read legacy version
        self.legacy_file_version = self.read_i32::<LE>()?;
        if self.legacy_file_version != -4 {
            // LegacyUE3Version for backwards-compatibility with UE3 games: always 864 in versioned assets, always 0 in unversioned assets
            self.read_exact(&mut [0u8; 4])?;
        }

        // read unreal version
        let file_version = self.read_i32::<LE>()?.try_into()?;

        self.asset_data.summary.unversioned = file_version == ObjectVersion::UNKNOWN;

        if self.asset_data.summary.unversioned {
            if self.asset_data.object_version == ObjectVersion::UNKNOWN {
                return Err(Error::invalid_file("Cannot begin serialization of an unversioned asset before an engine version is manually specified".to_string()));
            }
        } else {
            self.asset_data.object_version = file_version;
        }

        if self.legacy_file_version <= -8 {
            let object_version_ue5: ObjectVersionUE5 = self.read_i32::<LE>()?.try_into()?;
            if object_version_ue5 > ObjectVersionUE5::UNKNOWN {
                self.asset_data.object_version_ue5 = object_version_ue5;
            }
        }

        if self.asset_data.object_version_ue5 == ObjectVersionUE5::UNKNOWN {
            let mappings_version = self
                .get_mappings()
                .map(|e| e.object_version_ue5)
                .unwrap_or(ObjectVersionUE5::UNKNOWN);
            if mappings_version > ObjectVersionUE5::UNKNOWN {
                self.asset_data.object_version_ue5 = mappings_version;
            }
        }

        // read file license version
        self.asset_data.summary.file_licensee_version = self.read_i32::<LE>()?;

        // read custom versions container
        if self.legacy_file_version <= -2 {
            // TODO: support for enum-based custom versions
            let old_container = self.asset_data.summary.custom_versions.clone();
            self.asset_data.summary.custom_versions = self.read_custom_version_container(
                self.get_custom_version_serialization_format(),
                Some(&old_container),
            )?;
        }

        // read header offset
        self.header_offset = self.read_i32::<LE>()?;

        // read folder name
        self.folder_name = self
            .read_fstring()?
            .ok_or_else(|| Error::no_data("folder_name is None".to_string()))?;

        // read package flags
        self.asset_data.summary.package_flags = EPackageFlags::from_bits(self.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid package flags".to_string()))?;

        // read name count and offset
        self.name_count = self.read_i32::<LE>()?;
        self.name_offset = self.read_i32::<LE>()?;

        if self.get_object_version_ue5() >= ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST {
            self.soft_object_paths_count = self.read_i32::<LE>()?;
            self.soft_object_paths_offset = self.read_i32::<LE>()?;
        }

        // read text gatherable data
        if self.asset_data.object_version >= ObjectVersion::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES {
            self.gatherable_text_data_count = self.read_i32::<LE>()?;
            self.gatherable_text_data_offset = self.read_i32::<LE>()?;
        }

        // read count and offset for exports, imports, depends, soft package references, searchable names, thumbnail table
        self.asset_data.summary.export_count = self.read_i32::<LE>()?;
        self.export_offset = self.read_i32::<LE>()?;
        self.asset_data.summary.import_count = self.read_i32::<LE>()?;
        self.import_offset = self.read_i32::<LE>()?;
        self.depends_offset = self.read_i32::<LE>()?;
        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP
        {
            self.soft_package_reference_count = self.read_i32::<LE>()?;
            self.soft_package_reference_offset = self.read_i32::<LE>()?;
        }
        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES {
            self.searchable_names_offset = self.read_i32::<LE>()?;
        }
        self.thumbnail_table_offset = self.read_i32::<LE>()?;

        // read guid
        self.package_guid = self.raw_reader.read_guid()?;

        // raed generations
        let generations_count = self.read_i32::<LE>()?;
        for _ in 0..generations_count {
            let export_count = self.read_i32::<LE>()?;
            let name_count = self.read_i32::<LE>()?;
            self.generations.push(GenerationInfo {
                export_count,
                name_count,
            });
        }

        // read advanced engine version
        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ENGINE_VERSION_OBJECT {
            self.engine_version_recorded = FEngineVersion::read(self)?;
        } else {
            self.engine_version_recorded =
                FEngineVersion::new(4, 0, 0, self.read_u32::<LE>()?, None);
        }
        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
        {
            self.engine_version_compatible = FEngineVersion::read(self)?;
        } else {
            self.engine_version_compatible = self.engine_version_recorded.clone();
        }

        // read compression data
        self.compression_flags = self.read_u32::<LE>()?;
        let compression_block_count = self.read_u32::<LE>()?;
        if compression_block_count > 0 {
            return Err(Error::invalid_file(
                "Compression block count is not zero".to_string(),
            ));
        }

        self.package_source = self.read_u32::<LE>()?;

        // some other old unsupported stuff
        let additional_to_cook = self.read_i32::<LE>()?;
        if additional_to_cook != 0 {
            return Err(Error::invalid_file(
                "Additional to cook is not zero".to_string(),
            ));
        }
        if self.legacy_file_version > -7 {
            let texture_allocations_count = self.read_i32::<LE>()?;
            if texture_allocations_count != 0 {
                return Err(Error::invalid_file(
                    "Texture allocations count is not zero".to_string(),
                ));
            }
        }

        self.asset_registry_data_offset = self.read_i32::<LE>()?;
        self.bulk_data_start_offset = self.read_i64::<LE>()?;

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO {
            self.world_tile_info_offset = self.read_i32::<LE>()?;
        }

        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS
        {
            let chunk_id_count = self.read_i32::<LE>()?;

            for _ in 0..chunk_id_count {
                let chunk_id = self.read_i32::<LE>()?;
                self.chunk_ids.push(chunk_id);
            }
        } else if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE
        {
            self.chunk_ids = vec![];
            self.chunk_ids[0] = self.read_i32::<LE>()?;
        }

        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            self.preload_dependency_count = self.read_i32::<LE>()?;
            self.preload_dependency_offset = self.read_i32::<LE>()?;
        }

        self.names_referenced_from_export_data_count = match self.get_object_version_ue5()
            >= ObjectVersionUE5::NAMES_REFERENCED_FROM_EXPORT_DATA
        {
            true => self.read_i32::<LE>()?,
            false => self.name_count,
        };

        if self.get_object_version_ue5() >= ObjectVersionUE5::PAYLOAD_TOC {
            self.payload_toc_offset = self.read_i64::<LE>()?;
        }

        if self.get_object_version_ue5() >= ObjectVersionUE5::DATA_RESOURCES {
            self.data_resource_offset = self.read_i32::<LE>()?;
        }

        Ok(())
    }

    /// Get name map
    /// This method should be used if you want to mutate the namemap
    ///
    /// # Panics
    ///
    /// If the name map is borrowed mutably and you try to write the asset, the lib will panic
    pub fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    /// Search an FName reference
    pub fn search_name_reference(&self, name: &str) -> Option<i32> {
        self.name_map.get_ref().search_name_reference(name)
    }

    /// Add an FName reference
    pub fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
        self.name_map
            .get_mut()
            .add_name_reference(name, force_add_duplicates)
    }

    /// Get a name reference by an FName map index
    pub fn get_name_reference<T>(&self, index: i32, func: impl FnOnce(&str) -> T) -> T {
        func(self.name_map.get_ref().get_name_reference(index))
    }

    /// Add an `FName`
    pub fn add_fname(&mut self, slice: &str) -> FName {
        self.name_map.get_mut().add_fname(slice)
    }

    /// Add an `Import`
    pub fn add_import(&mut self, import: Import) -> PackageIndex {
        let index = -(self.imports.len() as i32) - 1;
        self.imports.push(import);
        PackageIndex::new(index)
    }

    /// Find an import, FName comparison is content-based
    pub fn find_import(
        &self,
        class_package: &FName,
        class_name: &FName,
        outer_index: PackageIndex,
        object_name: &FName,
    ) -> Option<i32> {
        for i in 0..self.imports.len() {
            let import = &self.imports[i];
            if import.class_package.eq_content(class_package)
                && import.class_name.eq_content(class_name)
                && import.outer_index == outer_index
                && import.object_name.eq_content(object_name)
            {
                return Some(-(i as i32) - 1);
            }
        }
        None
    }

    /// Find an import without specifying outer index, FName comparison is content-based
    pub fn find_import_no_index(
        &self,
        class_package: &FName,
        class_name: &FName,
        object_name: &FName,
    ) -> Option<i32> {
        for i in 0..self.imports.len() {
            let import = &self.imports[i];
            if import.class_package.eq_content(class_package)
                && import.class_name.eq_content(class_name)
                && import.object_name.eq_content(object_name)
            {
                return Some(-(i as i32) - 1);
            }
        }
        None
    }

    /// Get an import by [`PackageIndex`]
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

    /// Get an export
    pub fn get_export(&'a self, index: PackageIndex) -> Option<&'a Export<PackageIndex>> {
        self.asset_data.get_export(index)
    }

    /// Get a mutable export reference
    pub fn get_export_mut(
        &'a mut self,
        index: PackageIndex,
    ) -> Option<&'a mut Export<PackageIndex>> {
        self.asset_data.get_export_mut(index)
    }

    /// Get custom version serialization format
    pub fn get_custom_version_serialization_format(&self) -> ECustomVersionSerializationFormat {
        if self.legacy_file_version > 3 {
            return ECustomVersionSerializationFormat::Enums;
        }
        if self.legacy_file_version > -6 {
            return ECustomVersionSerializationFormat::Guids;
        }
        ECustomVersionSerializationFormat::Optimized
    }

    /// Parse asset data
    fn parse_data(&mut self) -> Result<(), Error> {
        self.parse_header()?;

        self.seek(SeekFrom::Start(self.name_offset as u64))?;

        for _ in 0..self.name_count {
            let (name, hash) = self.read_name_map_string(None)?;
            if hash == 0 {
                // todo: good FString type
                self.override_name_map_hashes.insert(name.clone(), 0);
            }
            self.add_name_reference(name, true);
        }

        if self.import_offset > 0 {
            self.seek(SeekFrom::Start(self.import_offset as u64))?;
            for _i in 0..self.asset_data.summary.import_count {
                let class_package = self.read_fname()?;
                let class_name = self.read_fname()?;
                let outer_index = PackageIndex::new(self.read_i32::<LE>()?);
                let object_name = self.read_fname()?;
                let optional =
                    match self.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
                        true => self.read_i32::<LE>()? == 1,
                        false => false,
                    };

                let import = Import::new(
                    class_package,
                    class_name,
                    outer_index,
                    object_name,
                    optional,
                );
                self.imports.push(import);
            }
        }

        let mut export_map = Vec::with_capacity(self.asset_data.summary.export_count as usize);
        if self.export_offset > 0 {
            self.seek(SeekFrom::Start(self.export_offset as u64))?;
            for _i in 0..self.asset_data.summary.export_count {
                export_map.push(UAssetExportMapEntry::read(self)?);
            }
        }

        let depends_offset_zero_version_range =
            ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
                ..ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES;
        if self.depends_offset > 0
            || depends_offset_zero_version_range.contains(&self.get_object_version())
        {
            let mut depends_map = Vec::with_capacity(self.asset_data.summary.export_count as usize);

            // 4.14-4.15 the depends offset wasnt updated so always serialized as 0
            if self.depends_offset > 0 {
                self.seek(SeekFrom::Start(self.depends_offset as u64))?;
            }

            for _i in 0..self.asset_data.summary.export_count as usize {
                let size = self.read_i32::<LE>()?;
                let mut data: Vec<i32> = Vec::new();
                for _j in 0..size {
                    data.push(self.read_i32::<LE>()?);
                }
                depends_map.push(data);
            }
            self.depends_map = Some(depends_map);
        }

        if self.soft_package_reference_offset > 0 {
            let mut soft_package_reference_list =
                Vec::with_capacity(self.soft_package_reference_count as usize);

            self.seek(SeekFrom::Start(self.soft_package_reference_offset as u64))?;

            for _i in 0..self.soft_package_reference_count as usize {
                if let Some(reference) = self.read_fstring()? {
                    soft_package_reference_list.push(reference);
                }
            }
            self.soft_package_reference_list = Some(soft_package_reference_list);
        }

        // TODO: Asset registry data parsing should be here

        if self.world_tile_info_offset > 0 {
            self.seek(SeekFrom::Start(self.world_tile_info_offset as u64))?;
            self.asset_data.world_tile_info = Some(FWorldTileInfo::new(self)?);
        }

        if self.asset_data.use_event_driven_loader {
            for entry in &mut export_map {
                self.raw_reader
                    .seek(SeekFrom::Start(self.preload_dependency_offset as u64))?;
                self.raw_reader.seek(SeekFrom::Current(
                    entry.first_export_dependency_offset as i64 * size_of::<i32>() as i64,
                ))?;

                let mut read_deps = |list: &mut Vec<PackageIndex>| -> Result<(), Error> {
                    for _ in 0..list.capacity() {
                        list.push(PackageIndex::new(self.raw_reader.read_i32::<LE>()?))
                    }
                    Ok(())
                };
                read_deps(&mut entry.serialization_before_serialization_dependencies)?;
                read_deps(&mut entry.create_before_serialization_dependencies)?;
                read_deps(&mut entry.serialization_before_create_dependencies)?;
                read_deps(&mut entry.create_before_create_dependencies)?;
            }
            self.seek(SeekFrom::Start(self.preload_dependency_offset as u64))?;
        }

        if self.header_offset > 0 && !export_map.is_empty() {
            let map_len = export_map.len();
            self.asset_data.exports.reserve(map_len);

            let serial_offsets = export_map
                .iter()
                .map(|e| e.serial_offset as u64)
                .collect::<Vec<_>>();

            for (i, entry) in export_map.into_iter().enumerate() {
                let base_export = entry.to_base_export();

                let next_starting = match i < (map_len - 1) {
                    true => serial_offsets[i + 1],
                    false => self.data_length()? - 4,
                };

                let export = self.read_export(base_export, next_starting)?;
                self.asset_data.exports.push(export);
            }
        }

        Ok(())
    }

    /// Write asset header
    fn write_header<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        cursor: &mut Writer,
        asset_header: &AssetHeader,
    ) -> Result<(), Error> {
        cursor.write_u32::<BE>(UE4_ASSET_MAGIC)?;
        cursor.write_i32::<LE>(self.legacy_file_version)?;

        if self.legacy_file_version != 4 {
            match self.asset_data.summary.unversioned {
                true => cursor.write_i32::<LE>(0)?,
                false => cursor.write_i32::<LE>(864)?,
            };
        }

        match self.asset_data.summary.unversioned {
            true => cursor.write_i32::<LE>(0)?,
            false => cursor.write_i32::<LE>(self.asset_data.object_version as i32)?,
        };

        if self.legacy_file_version <= -8 {
            match self.asset_data.summary.unversioned {
                true => cursor.write_i32::<LE>(0)?,
                false => cursor.write_i32::<LE>(self.get_object_version_ue5() as i32)?,
            };
        }

        cursor.write_i32::<LE>(self.asset_data.summary.file_licensee_version)?;
        if self.legacy_file_version <= -2 {
            match self.asset_data.summary.unversioned {
                true => cursor.write_i32::<LE>(0)?,
                false => {
                    cursor.write_i32::<LE>(self.asset_data.summary.custom_versions.len() as i32)?;
                    for custom_version in &self.asset_data.summary.custom_versions {
                        cursor.write_guid(&custom_version.guid)?;
                        cursor.write_i32::<LE>(custom_version.version)?;
                    }
                }
            };
        }

        cursor.write_i32::<LE>(asset_header.header_offset)?;
        cursor.write_fstring(Some(&self.folder_name))?;
        cursor.write_u32::<LE>(self.asset_data.summary.package_flags.bits())?;
        cursor.write_i32::<LE>(self.name_map.get_ref().get_name_map_index_list().len() as i32)?;
        cursor.write_i32::<LE>(asset_header.name_offset)?;

        if self.get_object_version_ue5() >= ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST {
            cursor.write_i32::<LE>(self.soft_object_paths_count)?;
            cursor.write_i32::<LE>(self.soft_object_paths_offset)?;
        }

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES {
            cursor.write_i32::<LE>(self.gatherable_text_data_count)?;
            cursor.write_i32::<LE>(self.gatherable_text_data_offset)?;
        }

        cursor.write_i32::<LE>(self.asset_data.exports.len() as i32)?;
        cursor.write_i32::<LE>(asset_header.export_offset)?;
        cursor.write_i32::<LE>(self.imports.len() as i32)?;
        cursor.write_i32::<LE>(asset_header.import_offset)?;
        cursor.write_i32::<LE>(asset_header.depends_offset)?;

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP
        {
            cursor.write_i32::<LE>(self.soft_package_reference_count)?;
            cursor.write_i32::<LE>(asset_header.soft_package_reference_offset)?;
        }

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES {
            cursor.write_i32::<LE>(self.searchable_names_offset)?;
        }

        cursor.write_i32::<LE>(self.thumbnail_table_offset)?;
        cursor.write_guid(&self.package_guid)?;
        cursor.write_i32::<LE>(self.generations.len() as i32)?;

        for _ in 0..self.generations.len() {
            cursor.write_i32::<LE>(self.asset_data.exports.len() as i32)?;
            cursor
                .write_i32::<LE>(self.name_map.get_ref().get_name_map_index_list().len() as i32)?;
        }

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_ENGINE_VERSION_OBJECT {
            self.engine_version_recorded.write(cursor)?;
        } else {
            cursor.write_u32::<LE>(self.engine_version_recorded.build)?;
        }

        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
        {
            self.engine_version_recorded.write(cursor)?;
        }

        cursor.write_u32::<LE>(self.compression_flags)?;
        cursor.write_i32::<LE>(0)?; // numCompressedChunks
        cursor.write_u32::<LE>(self.package_source)?;
        cursor.write_i32::<LE>(0)?; // numAdditionalPackagesToCook

        if self.legacy_file_version > -7 {
            cursor.write_i32::<LE>(0)?; // numTextureallocations
        }

        cursor.write_i32::<LE>(asset_header.asset_registry_data_offset)?;
        cursor.write_i64::<LE>(asset_header.bulk_data_start_offset)?;

        if self.asset_data.object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO {
            cursor.write_i32::<LE>(asset_header.world_tile_info_offset)?;
        }

        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS
        {
            cursor.write_i32::<LE>(self.chunk_ids.len() as i32)?;
            for chunk_id in &self.chunk_ids {
                cursor.write_i32::<LE>(*chunk_id)?;
            }
        } else if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE
        {
            cursor.write_i32::<LE>(self.chunk_ids[0])?;
        }

        if self.asset_data.object_version
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            cursor.write_i32::<LE>(asset_header.preload_dependency_count)?;
            cursor.write_i32::<LE>(asset_header.preload_dependency_offset)?;
        }

        if self.get_object_version_ue5() >= ObjectVersionUE5::NAMES_REFERENCED_FROM_EXPORT_DATA {
            cursor.write_i32::<LE>(self.names_referenced_from_export_data_count)?;
        }

        if self.get_object_version_ue5() >= ObjectVersionUE5::PAYLOAD_TOC {
            cursor.write_i64::<LE>(self.payload_toc_offset)?;
        }

        if self.get_object_version_ue5() >= ObjectVersionUE5::DATA_RESOURCES {
            cursor.write_i32::<LE>(self.data_resource_offset)?;
        }

        Ok(())
    }

    /// Rebuild the FName map
    /// This can be used if it's too complicated to keep track of all FNames that were added into the asset
    /// This is useful when copying export from one asset into another
    /// This will automatically figure out every new FName and add them to the name map
    pub fn rebuild_name_map(&mut self) {
        let mut current_name_map = self.name_map.clone();
        self.traverse_fnames(&mut |mut name| {
            let content = name.get_owned_content();
            let FName::Backed {
                index, name_map, ..
            } = &mut name
            else {
                return;
            };

            if *name_map != current_name_map {
                let new_index = current_name_map
                    .get_mut()
                    .add_name_reference(content, false);

                *index = new_index;
                *name_map = current_name_map.clone();
            }
        });
    }

    /// Write asset data
    pub fn write_data<W: Read + Seek + Write>(
        &self,
        cursor: &mut W,
        uexp_cursor: Option<&mut W>,
    ) -> Result<(), Error> {
        if self.asset_data.use_event_driven_loader != uexp_cursor.is_some() {
            return Err(Error::no_data(format!(
                "use_separate_bulk_data_files is {} but uexp_cursor is {}",
                self.asset_data.use_event_driven_loader,
                match uexp_cursor.is_some() {
                    true => "Some(...)",
                    false => "None",
                }
            )));
        }

        let header = AssetHeader {
            name_offset: self.name_offset,
            import_offset: self.import_offset,
            export_offset: self.export_offset,
            depends_offset: self.depends_offset,
            soft_package_reference_offset: self.soft_package_reference_offset,
            asset_registry_data_offset: self.asset_registry_data_offset,
            world_tile_info_offset: self.world_tile_info_offset,
            preload_dependency_count: 0,
            preload_dependency_offset: self.preload_dependency_offset,
            header_offset: self.header_offset,
            bulk_data_start_offset: self.bulk_data_start_offset,
        };

        let mut raw_serializer = RawWriter::new(
            cursor,
            self.asset_data.object_version,
            self.asset_data.object_version_ue5,
            self.asset_data.use_event_driven_loader,
            self.name_map.clone(),
        );
        let mut serializer = AssetArchiveWriter::new(
            &mut raw_serializer,
            &self.asset_data,
            &self.imports,
            self.name_map.clone(),
        );

        self.write_header(&mut serializer, &header)?;

        let name_offset = match !self.name_map.get_ref().is_empty() {
            true => serializer.position() as i32,
            false => 0,
        };

        for name in self.name_map.get_ref().get_name_map_index_list() {
            // todo: case preserving FString
            serializer.write_fstring(Some(name))?;

            if self.asset_data.object_version >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
                match self.override_name_map_hashes.get_by_key(name) {
                    Some(e) => serializer.write_u32::<LE>(*e)?,
                    None => serializer.write_u32::<LE>(crc::generate_hash(name))?,
                };
            }
        }

        let import_offset = match !self.imports.is_empty() {
            true => serializer.position() as i32,
            false => 0,
        };

        for import in &self.imports {
            serializer.write_fname(&import.class_package)?;
            serializer.write_fname(&import.class_name)?;
            serializer.write_i32::<LE>(import.outer_index.index)?;
            serializer.write_fname(&import.object_name)?;
            if serializer.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
                serializer.write_i32::<LE>(match import.optional {
                    true => 1,
                    false => 0,
                })?;
            }
        }

        let export_offset = match !self.asset_data.exports.is_empty() {
            true => serializer.position() as i32,
            false => 0,
        };

        for export in &self.asset_data.exports {
            let unk: &BaseExport<PackageIndex> = export.get_base_export();
            UAssetExportMapEntry::from_base_export(unk).write(
                &mut serializer,
                unk.serial_size,
                unk.serial_offset,
                unk.first_export_dependency_offset,
            )?;
        }

        let depends_offset = match self.depends_map {
            Some(_) => serializer.position() as i32,
            None => 0,
        };

        if let Some(ref map) = self.depends_map {
            for i in 0..self.asset_data.exports.len() {
                let dummy = Vec::new();
                let current_data = match map.get(i) {
                    Some(e) => e,
                    None => &dummy,
                };
                serializer.write_i32::<LE>(current_data.len() as i32)?;
                for i in current_data {
                    serializer.write_i32::<LE>(*i)?;
                }
            }
        }

        let soft_package_reference_offset = match self.soft_package_reference_list {
            Some(_) => serializer.position() as i32,
            None => 0,
        };

        if let Some(ref package_references) = self.soft_package_reference_list {
            for reference in package_references {
                serializer.write_fstring(Some(reference))?;
            }
        }

        // todo: asset registry data support
        // we can support it now I think?
        let asset_registry_data_offset = match self.asset_registry_data_offset != 0 {
            true => serializer.position() as i32,
            false => 0,
        };
        if self.asset_registry_data_offset != 0 {
            serializer.write_i32::<LE>(0)?; // asset registry data length
        }

        let world_tile_info_offset = match self.asset_data.world_tile_info {
            Some(_) => serializer.position() as i32,
            None => 0,
        };

        if let Some(ref world_tile_info) = self.asset_data.world_tile_info {
            world_tile_info.write(&mut serializer)?;
        }

        let mut preload_dependency_count = 0;
        let preload_dependency_offset = serializer.position() as i32;

        if self.asset_data.use_event_driven_loader {
            for export in &self.asset_data.exports {
                let unk_export = export.get_base_export();

                for element in &unk_export.serialization_before_serialization_dependencies {
                    serializer.write_i32::<LE>(element.index)?;
                }

                for element in &unk_export.create_before_serialization_dependencies {
                    serializer.write_i32::<LE>(element.index)?;
                }

                for element in &unk_export.serialization_before_create_dependencies {
                    serializer.write_i32::<LE>(element.index)?;
                }

                for element in &unk_export.create_before_create_dependencies {
                    serializer.write_i32::<LE>(element.index)?;
                }

                preload_dependency_count += unk_export
                    .serialization_before_serialization_dependencies
                    .len() as i32
                    + unk_export.create_before_serialization_dependencies.len() as i32
                    + unk_export.serialization_before_create_dependencies.len() as i32
                    + unk_export.create_before_create_dependencies.len() as i32;
            }
        } else {
            preload_dependency_count = -1;
        }

        let header_offset = match !self.asset_data.exports.is_empty() {
            true => serializer.position() as i32,
            false => 0,
        };

        let mut category_starts = Vec::with_capacity(self.asset_data.exports.len());

        let final_cursor_pos = serializer.position();

        let mut raw_bulk_serializer = match self.asset_data.use_event_driven_loader {
            true => Some(RawWriter::new(
                uexp_cursor.unwrap(),
                self.asset_data.object_version,
                self.asset_data.object_version_ue5,
                self.asset_data.use_event_driven_loader,
                self.name_map.clone(),
            )),
            false => None,
        };

        let mut bulk_serializer = match self.asset_data.use_event_driven_loader {
            true => Some(AssetArchiveWriter::new(
                raw_bulk_serializer.as_mut().unwrap(),
                &self.asset_data,
                &self.imports,
                self.name_map.clone(),
            )),
            false => None,
        };

        let bulk_serializer = match self.asset_data.use_event_driven_loader {
            true => bulk_serializer.as_mut().unwrap(),
            false => &mut serializer,
        };

        for export in &self.asset_data.exports {
            category_starts.push(match self.asset_data.use_event_driven_loader {
                true => bulk_serializer.position() + final_cursor_pos,
                false => bulk_serializer.position(),
            });

            export.write(bulk_serializer)?;

            if let Some(normal_export) = export.get_normal_export() {
                bulk_serializer.write_all(&normal_export.extras)?;
            }
        }
        bulk_serializer.write_all(&[0xc1, 0x83, 0x2a, 0x9e])?;

        let bulk_data_start_offset = match self.asset_data.use_event_driven_loader {
            true => final_cursor_pos as i64 + bulk_serializer.position() as i64,
            false => bulk_serializer.position() as i64,
        } - 4;

        bulk_serializer.rewind()?;

        if !self.asset_data.exports.is_empty() {
            serializer.seek(SeekFrom::Start(export_offset as u64))?;
            let mut first_export_dependency_offset = 0;
            for i in 0..self.asset_data.exports.len() {
                let unk = &self.asset_data.exports[i].get_base_export();
                let next_loc = match self.asset_data.exports.len() - 1 > i {
                    true => category_starts[i + 1] as i64,
                    false => bulk_data_start_offset,
                };

                UAssetExportMapEntry::from_base_export(unk).write(
                    &mut serializer,
                    next_loc - category_starts[i] as i64,
                    category_starts[i] as i64,
                    match self.asset_data.use_event_driven_loader {
                        true => first_export_dependency_offset,
                        false => -1,
                    },
                )?;

                first_export_dependency_offset +=
                    (unk.serialization_before_serialization_dependencies.len()
                        + unk.create_before_serialization_dependencies.len()
                        + unk.serialization_before_create_dependencies.len()
                        + unk.create_before_create_dependencies.len()) as i32;
            }
        }

        serializer.rewind()?;

        let header = AssetHeader {
            name_offset,
            import_offset,
            export_offset,
            depends_offset,
            soft_package_reference_offset,
            asset_registry_data_offset,
            world_tile_info_offset,
            preload_dependency_count,
            preload_dependency_offset,
            header_offset,
            bulk_data_start_offset,
        };
        self.write_header(&mut serializer, &header)?;

        serializer.rewind()?;

        Ok(())
    }
}

impl<C: Read + Seek> AssetTrait<PackageIndex> for Asset<C> {
    fn get_asset_data(&self) -> &AssetData<PackageIndex> {
        &self.asset_data
    }

    fn get_asset_data_mut(&mut self) -> &mut AssetData<PackageIndex> {
        &mut self.asset_data
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn search_name_reference(&self, name: &str) -> Option<i32> {
        self.name_map.get_ref().search_name_reference(name)
    }

    fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
        self.name_map
            .get_mut()
            .add_name_reference(name, force_add_duplicates)
    }

    fn get_name_reference<T>(&self, index: i32, func: impl FnOnce(&str) -> T) -> T {
        func(self.name_map.get_ref().get_name_reference(index))
    }

    fn add_fname(&mut self, slice: &str) -> FName {
        self.name_map.get_mut().add_fname(slice)
    }
}

impl<C: Read + Seek> ArchiveTrait<PackageIndex> for Asset<C> {
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
        self.raw_reader.position()
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

impl<C: Read + Seek> ArchiveReader<PackageIndex> for Asset<C> {
    passthrough_archive_reader!(raw_reader);
}

impl<C: Read + Seek> Read for Asset<C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.raw_reader.read(buf)
    }
}

impl<C: Read + Seek> Seek for Asset<C> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.raw_reader.seek(pos)
    }
}

// custom debug implementation to not print the whole data buffer
impl<C: Read + Seek> Debug for Asset<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Asset")
            .field("info", &self.info)
            .field("asset_data", &self.asset_data)
            .field("legacy_file_version", &self.legacy_file_version)
            // imports
            // exports
            // depends map
            // soft package reference list
            // asset registry data
            // world tile info
            // preload dependencies
            .field("generations", &self.generations)
            .field("package_guid", &self.package_guid)
            .field("engine_version", &self.asset_data.get_engine_version())
            .field("engine_version_recorded", &self.engine_version_recorded)
            .field("engine_version_compatible", &self.engine_version_compatible)
            .field("chunk_ids", &self.chunk_ids)
            .field("asset_data", &self.asset_data)
            .field("package_source", &self.package_source)
            .field("folder_name", &self.folder_name)
            // map struct type override
            // override name map hashes
            .field("header_offset", &self.header_offset)
            .field("name_count", &self.name_count)
            .field("name_offset", &self.name_offset)
            .field(
                "gatherable_text_data_count",
                &self.gatherable_text_data_count,
            )
            .field(
                "gatherable_text_data_offset",
                &self.gatherable_text_data_offset,
            )
            .field("export_offset", &self.export_offset)
            .field("import_offset", &self.import_offset)
            .field("depends_offset", &self.depends_offset)
            .field(
                "soft_package_reference_count",
                &self.soft_package_reference_count,
            )
            .field(
                "soft_package_reference_offset",
                &self.soft_package_reference_offset,
            )
            .field("searchable_names_offset", &self.searchable_names_offset)
            .field("thumbnail_table_offset", &self.thumbnail_table_offset)
            .field("compression_flags", &self.compression_flags)
            .field(
                "asset_registry_data_offset",
                &self.asset_registry_data_offset,
            )
            .field("bulk_data_start_offset", &self.bulk_data_start_offset)
            .field("world_tile_info_data_offset", &self.world_tile_info_offset)
            .field("preload_dependency_count", &self.preload_dependency_count)
            .field("preload_dependency_offset", &self.preload_dependency_offset)
            .finish()
    }
}
