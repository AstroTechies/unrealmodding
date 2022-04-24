use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;

use std::fmt::{Debug, Formatter};

use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::exports::class_export::ClassExport;
use crate::exports::data_table_export::DataTableExport;
use crate::exports::enum_export::EnumExport;
use crate::exports::level_export::LevelExport;
use crate::exports::normal_export::NormalExport;
use crate::exports::property_export::PropertyExport;
use crate::exports::raw_export::RawExport;
use crate::exports::string_table_export::StringTableExport;
use crate::exports::struct_export::StructExport;
use crate::exports::unknown_export::UnknownExport;
use crate::exports::{ExportTrait, ExportUnknownTrait};
use crate::fproperty::FProperty;
use crate::ue4version::{
    VER_UE4_64BIT_EXPORTMAP_SERIALSIZES, VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE,
    VER_UE4_ADDED_SEARCHABLE_NAMES, VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP,
    VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS, VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT,
    VER_UE4_ENGINE_VERSION_OBJECT, VER_UE4_LOAD_FOR_EDITOR_GAME, VER_UE4_NAME_HASHES_SERIALIZED,
    VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION,
    VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS, VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG,
    VER_UE4_SERIALIZE_TEXT_IN_PACKAGES, VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS,
    VER_UE4_WORLD_LEVEL_INFO,
};

use self::cursor_ext::CursorExt;
use self::custom_version::CustomVersionTrait;
use self::error::Error;
use self::exports::{Export, ExportNormalTrait};

use self::properties::world_tile_property::FWorldTileInfo;
use self::unreal_types::Guid;

mod crc;
pub mod cursor_ext;
pub mod custom_version;
pub mod enums;
pub mod error;
pub mod exports;
pub mod flags;
pub mod fproperty;
pub mod kismet;
pub mod properties;
pub mod types;
pub mod ue4version;
pub mod unreal_types;
pub mod uproperty;
use custom_version::CustomVersion;
use unreal_types::{FName, GenerationInfo};

#[derive(Debug, Clone)]
pub struct Import {
    pub class_package: FName,
    pub class_name: FName,
    pub outer_index: i32,
    pub object_name: FName,
}

impl Import {
    pub fn new(
        class_package: FName,
        class_name: FName,
        outer_index: i32,
        object_name: FName,
    ) -> Self {
        Import {
            class_package,
            class_name,
            object_name,
            outer_index,
        }
    }
}

const UE4_ASSET_MAGIC: u32 = u32::from_be_bytes([0xc1, 0x83, 0x2a, 0x9e]);

//#[derive(Debug)]
pub struct Asset {
    // raw data
    pub(crate) cursor: Cursor<Vec<u8>>,
    data_length: u64,

    // parsed data
    pub info: String,
    pub use_separate_bulk_data_files: bool,
    pub engine_version: i32,
    pub legacy_file_version: i32,
    pub unversioned: bool,
    pub file_license_version: i32,
    pub custom_version: Vec<CustomVersion>,
    // imports
    // exports
    // depends map
    // soft package reference list
    // asset registry data
    // world tile info
    // preload dependencies
    pub generations: Vec<GenerationInfo>,
    pub package_guid: Guid,
    pub engine_version_recorded: EngineVersion,
    pub engine_version_compatible: EngineVersion,
    chunk_ids: Vec<i32>,
    pub package_flags: u32,
    pub package_source: u32,
    pub folder_name: String,
    // map struct type override
    // override name map hashes
    header_offset: i32,
    name_count: i32,
    name_offset: i32,
    gatherable_text_data_count: i32,
    gatherable_text_data_offset: i32,
    export_count: i32,
    export_offset: i32,
    import_count: i32,
    import_offset: i32,
    depends_offset: i32,
    soft_package_reference_count: i32,
    soft_package_reference_offset: i32,
    searchable_names_offset: i32,
    thumbnail_table_offset: i32,
    compression_flags: u32,
    asset_registry_data_offset: i32,
    bulk_data_start_offset: i64,
    world_tile_info_offset: i32,
    preload_dependency_count: i32,
    preload_dependency_offset: i32,

    override_name_map_hashes: HashMap<String, u32>,
    name_map_index_list: Vec<String>,
    name_map_lookup: HashMap<u64, i32>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    depends_map: Option<Vec<Vec<i32>>>,
    soft_package_reference_list: Option<Vec<String>>,
    pub world_tile_info: Option<FWorldTileInfo>,
    preload_dependencies: Option<Vec<i32>>,

    //todo: fill out with defaults
    pub map_key_override: HashMap<String, String>,
    pub map_value_override: HashMap<String, String>,
}

impl<'a> Asset {
    pub fn new(asset_data: Vec<u8>, bulk_data: Option<Vec<u8>>) -> Self {
        let raw_data = match &bulk_data {
            Some(e) => {
                let mut data = asset_data.clone();
                data.extend(e);
                data
            }
            None => asset_data,
        };

        Asset {
            data_length: raw_data.len() as u64,
            cursor: Cursor::new(raw_data),
            info: String::from("Serialized with unrealmodding/uasset"),
            use_separate_bulk_data_files: bulk_data.is_some(),
            engine_version: 0,
            legacy_file_version: 0,
            unversioned: true,
            file_license_version: 0,
            custom_version: Vec::new(),
            generations: Vec::new(),
            package_guid: [0; 16],
            engine_version_recorded: EngineVersion::unknown(),
            engine_version_compatible: EngineVersion::unknown(),
            chunk_ids: Vec::new(),
            package_flags: 0,
            package_source: 0,
            folder_name: String::from(""),
            header_offset: 0,
            name_count: 0,
            name_offset: 0,
            gatherable_text_data_count: 0,
            gatherable_text_data_offset: 0,
            export_count: 0,
            export_offset: 0,
            import_count: 0,
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

            override_name_map_hashes: HashMap::new(),
            name_map_index_list: Vec::new(),
            name_map_lookup: HashMap::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            depends_map: None,
            soft_package_reference_list: None,
            world_tile_info: None,
            preload_dependencies: None,
            map_key_override: HashMap::new(), // todo: preinit
            map_value_override: HashMap::new(),
        }
    }

    fn parse_header(&mut self) -> Result<(), Error> {
        // reuseable buffers for reading

        // seek to start
        self.cursor.seek(SeekFrom::Start(0))?;

        // read and check magic
        if self.cursor.read_u32::<BigEndian>()? != UE4_ASSET_MAGIC {
            return Err(Error::invalid_file(
                "File is not a valid uasset file".to_string(),
            ));
        }

        // read legacy version
        self.legacy_file_version = self.cursor.read_i32::<LittleEndian>()?;
        if self.legacy_file_version != -4 {
            // LegacyUE3Version for backwards-compatibility with UE3 games: always 864 in versioned assets, always 0 in unversioned assets
            self.cursor.read_exact(&mut [0u8; 4])?;
        }

        // read unreal version
        let file_version = self.cursor.read_i32::<LittleEndian>()?;

        self.unversioned = file_version == ue4version::UNKNOWN;

        if self.unversioned {
            if self.engine_version == ue4version::UNKNOWN {
                return Err(Error::invalid_file("Cannot begin serialization of an unversioned asset before an engine version is manually specified".to_string()));
            }
        } else {
            self.engine_version = file_version;
        }

        // read file license version
        self.file_license_version = self.cursor.read_i32::<LittleEndian>()?;

        // read custom versions container
        if self.legacy_file_version <= -2 {
            // TODO: support for enum-based custom versions

            // read custom version count
            let custom_versions_count = self.cursor.read_i32::<LittleEndian>()?;

            for _ in 0..custom_versions_count {
                // read guid
                let mut guid = [0u8; 16];
                self.cursor.read_exact(&mut guid)?;
                // read version
                let version = self.cursor.read_i32::<LittleEndian>()?;

                self.custom_version.push(CustomVersion::new(guid, version));
            }
        }

        // read header offset
        self.header_offset = self.cursor.read_i32::<LittleEndian>()?;

        // read folder name
        self.folder_name = self
            .cursor
            .read_string()?
            .ok_or(Error::no_data("folder_name is None".to_string()))?;

        // read package flags
        self.package_flags = self.cursor.read_u32::<LittleEndian>()?;

        // read name count and offset
        self.name_count = self.cursor.read_i32::<LittleEndian>()?;
        self.name_offset = self.cursor.read_i32::<LittleEndian>()?;
        // read text gatherable data
        if self.engine_version >= ue4version::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES {
            self.gatherable_text_data_count = self.cursor.read_i32::<LittleEndian>()?;
            self.gatherable_text_data_offset = self.cursor.read_i32::<LittleEndian>()?;
        }

        // read count and offset for exports, imports, depends, soft package references, searchable names, thumbnail table
        self.export_count = self.cursor.read_i32::<LittleEndian>()?;
        self.export_offset = self.cursor.read_i32::<LittleEndian>()?;
        self.import_count = self.cursor.read_i32::<LittleEndian>()?;
        self.import_offset = self.cursor.read_i32::<LittleEndian>()?;
        self.depends_offset = self.cursor.read_i32::<LittleEndian>()?;
        if self.engine_version >= ue4version::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP {
            self.soft_package_reference_count = self.cursor.read_i32::<LittleEndian>()?;
            self.soft_package_reference_offset = self.cursor.read_i32::<LittleEndian>()?;
        }
        if self.engine_version >= ue4version::VER_UE4_ADDED_SEARCHABLE_NAMES {
            self.searchable_names_offset = self.cursor.read_i32::<LittleEndian>()?;
        }
        self.thumbnail_table_offset = self.cursor.read_i32::<LittleEndian>()?;

        // read guid
        self.cursor.read_exact(&mut self.package_guid)?;

        // raed generations
        let generations_count = self.cursor.read_i32::<LittleEndian>()?;
        for _ in 0..generations_count {
            let export_count = self.cursor.read_i32::<LittleEndian>()?;
            let name_count = self.cursor.read_i32::<LittleEndian>()?;
            self.generations.push(GenerationInfo {
                export_count,
                name_count,
            });
        }

        // read advanced engine version
        if self.engine_version >= ue4version::VER_UE4_ENGINE_VERSION_OBJECT {
            self.engine_version_recorded = EngineVersion::read(&mut self.cursor)?;
        } else {
            self.engine_version_recorded =
                EngineVersion::new(4, 0, 0, self.cursor.read_u32::<LittleEndian>()?, None);
        }
        if self.engine_version >= ue4version::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
        {
            self.engine_version_compatible = EngineVersion::read(&mut self.cursor)?;
        } else {
            self.engine_version_compatible = self.engine_version_recorded.clone();
        }

        // read compression data
        self.compression_flags = self.cursor.read_u32::<LittleEndian>()?;
        let compression_block_count = self.cursor.read_u32::<LittleEndian>()?;
        if compression_block_count > 0 {
            return Err(Error::invalid_file(
                "Compression block count is not zero".to_string(),
            ));
        }

        self.package_source = self.cursor.read_u32::<LittleEndian>()?;

        // some other old unsupported stuff
        let additional_to_cook = self.cursor.read_i32::<LittleEndian>()?;
        if additional_to_cook != 0 {
            return Err(Error::invalid_file(
                "Additional to cook is not zero".to_string(),
            ));
        }
        if self.legacy_file_version > -7 {
            let texture_allocations_count = self.cursor.read_i32::<LittleEndian>()?;
            if texture_allocations_count != 0 {
                return Err(Error::invalid_file(
                    "Texture allocations count is not zero".to_string(),
                ));
            }
        }

        self.asset_registry_data_offset = self.cursor.read_i32::<LittleEndian>()?;
        self.bulk_data_start_offset = self.cursor.read_i64::<LittleEndian>()?;

        if self.engine_version >= ue4version::VER_UE4_WORLD_LEVEL_INFO {
            self.world_tile_info_offset = self.cursor.read_i32::<LittleEndian>()?;
        }

        if self.engine_version >= ue4version::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS {
            let chunk_id_count = self.cursor.read_i32::<LittleEndian>()?;

            for _ in 0..chunk_id_count {
                let chunk_id = self.cursor.read_i32::<LittleEndian>()?;
                self.chunk_ids.push(chunk_id);
            }
        } else if self.engine_version >= ue4version::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE
        {
            self.chunk_ids = vec![];
            self.chunk_ids[0] = self.cursor.read_i32::<LittleEndian>()?;
        }

        if self.engine_version >= ue4version::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
            self.preload_dependency_count = self.cursor.read_i32::<LittleEndian>()?;
            self.preload_dependency_offset = self.cursor.read_i32::<LittleEndian>()?;
        }
        Ok(())
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        let version_friendly_name = T::friendly_name();
        for i in 0..self.custom_version.len() {
            if let Some(friendly_name) = &self.custom_version[i].friendly_name {
                if friendly_name.as_str() == version_friendly_name {
                    return self.custom_version[i].clone();
                }
            }
        }

        let guess = T::from_engine_version(self.engine_version);
        CustomVersion::from_version(guess)
    }

    fn read_name_map_string(&mut self) -> Result<(u32, String), Error> {
        let s = self
            .cursor
            .read_string()?
            .ok_or(Error::no_data("name_map_string is None".to_string()))?;
        let mut hashes = 0;
        if self.engine_version >= ue4version::VER_UE4_NAME_HASHES_SERIALIZED && !s.is_empty() {
            hashes = self.cursor.read_u32::<LittleEndian>()?;
        }
        Ok((hashes, s))
    }

    pub(crate) fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        if self.engine_version >= ue4version::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            let has_property_guid = self.cursor.read_bool()?;
            if has_property_guid {
                let mut guid = [0u8; 16];
                self.cursor.read_exact(&mut guid)?;
                return Ok(Some(guid));
            }
        }
        Ok(None)
    }

    pub(crate) fn write_property_guid(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        guid: &Option<Guid>,
    ) -> Result<(), Error> {
        if self.engine_version >= VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            cursor.write_bool(guid.is_some())?;
            if let Some(ref data) = guid {
                cursor.write(data)?;
            }
        }
        Ok(())
    }

    pub fn search_name_reference(&self, name: &String) -> Option<i32> {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);

        self.name_map_lookup.get(&s.finish()).map(|e| *e)
    }

    pub fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
        if !force_add_duplicates {
            let existing = self.search_name_reference(&name);
            if existing.is_some() {
                return existing.unwrap();
            }
        }

        let mut s = DefaultHasher::new();
        name.hash(&mut s);

        let hash = s.finish();
        self.name_map_index_list.push(name.clone());
        self.name_map_lookup
            .insert(hash, self.name_map_lookup.len() as i32);
        (self.name_map_lookup.len() - 1) as i32
    }

    pub fn get_name_reference(&self, index: i32) -> String {
        if index < 0 {
            return (-index).to_string(); // is this right even?
        }
        if index >= self.name_map_index_list.len() as i32 {
            return index.to_string();
        }
        self.name_map_index_list[index as usize].to_owned()
    }

    pub fn add_fname(&mut self, slice: &str) -> FName {
        let name = FName::from_slice(slice);
        self.add_name_reference(name.content.clone(), false);
        name
    }

    fn read_fname(&mut self) -> Result<FName, Error> {
        let name_map_pointer = self.cursor.read_i32::<LittleEndian>()?;
        let number = self.cursor.read_i32::<LittleEndian>()?;

        Ok(FName::new(
            self.get_name_reference(name_map_pointer),
            number,
        ))
    }

    fn write_fname(&self, cursor: &mut Cursor<Vec<u8>>, fname: &FName) -> Result<(), Error> {
        cursor.write_i32::<LittleEndian>(self.search_name_reference(&fname.content).ok_or(
            Error::no_data(format!(
                "name reference for {} not found",
                fname.content.to_owned()
            )),
        )?)?;
        cursor.write_i32::<LittleEndian>(fname.index)?;
        Ok(())
    }

    pub fn add_import(&mut self, import: Import) -> i32 {
        let index = -(self.imports.len() as i32) - 1;
        let import = import.clone();
        self.imports.push(import);
        index
    }

    pub fn get_import(&'a self, index: i32) -> Option<&'a Import> {
        if !is_import(index) {
            return None;
        }

        let index = -index - 1;
        if index < 0 || index > self.imports.len() as i32 {
            return None;
        }

        Some(&self.imports[index as usize])
    }

    pub fn get_export(&'a self, index: i32) -> Option<&'a Export> {
        if !is_export(index) {
            return None;
        }

        let index = index - 1;

        if index < 0 || index >= self.exports.len() as i32 {
            return None;
        }

        Some(&self.exports[index as usize])
    }

    pub fn get_export_mut(&'a mut self, index: i32) -> Option<&'a mut Export> {
        if !is_export(index) {
            return None;
        }

        let index = index - 1;

        if index < 0 || index >= self.exports.len() as i32 {
            return None;
        }

        Some(&mut self.exports[index as usize])
    }

    fn get_export_class_type(&'a self, index: i32) -> Option<FName> {
        match is_import(index) {
            true => self.get_import(index).map(|e| e.object_name.clone()),
            false => Some(FName::new(index.to_string(), 0)),
        }
    }

    pub fn parse_data(&mut self) -> Result<(), Error> {
        self.parse_header()?;
        self.cursor.seek(SeekFrom::Start(self.name_offset as u64))?;

        for _i in 0..self.name_count {
            let name_map = self.read_name_map_string()?;
            if name_map.0 == 0 {
                if let Some(entry) = self.override_name_map_hashes.get_mut(&name_map.1) {
                    *entry = 0u32;
                }
            }
            self.add_name_reference(name_map.1, true);
        }

        if self.import_offset > 0 {
            self.cursor
                .seek(SeekFrom::Start(self.import_offset as u64))?;
            for _i in 0..self.import_count {
                let import = Import::new(
                    self.read_fname()?,
                    self.read_fname()?,
                    self.cursor.read_i32::<LittleEndian>()?,
                    self.read_fname()?,
                );
                self.imports.push(import);
            }
        }

        if self.export_offset > 0 {
            self.cursor
                .seek(SeekFrom::Start(self.export_offset as u64))?;
            for _i in 0..self.export_count {
                let mut export = UnknownExport::default();
                export.class_index = self.cursor.read_i32::<LittleEndian>()?;
                export.super_index = self.cursor.read_i32::<LittleEndian>()?;

                if self.engine_version >= VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS {
                    export.template_index = self.cursor.read_i32::<LittleEndian>()?;
                }

                export.outer_index = self.cursor.read_i32::<LittleEndian>()?;
                export.object_name = self.read_fname()?;
                export.object_flags = self.cursor.read_u32::<LittleEndian>()?;

                if self.engine_version < VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
                    export.serial_size = self.cursor.read_i32::<LittleEndian>()? as i64;
                    export.serial_offset = self.cursor.read_i32::<LittleEndian>()? as i64;
                } else {
                    export.serial_size = self.cursor.read_i64::<LittleEndian>()?;
                    export.serial_offset = self.cursor.read_i64::<LittleEndian>()?;
                }

                export.forced_export = self.cursor.read_i32::<LittleEndian>()? == 1;
                export.not_for_client = self.cursor.read_i32::<LittleEndian>()? == 1;
                export.not_for_server = self.cursor.read_i32::<LittleEndian>()? == 1;
                self.cursor.read_exact(&mut export.package_guid)?;
                export.package_flags = self.cursor.read_u32::<LittleEndian>()?;

                if self.engine_version >= VER_UE4_LOAD_FOR_EDITOR_GAME {
                    export.not_always_loaded_for_editor_game =
                        self.cursor.read_i32::<LittleEndian>()? == 1;
                }

                if self.engine_version >= VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
                    export.is_asset = self.cursor.read_i32::<LittleEndian>()? == 1;
                }

                if self.engine_version >= VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
                    export.first_export_dependency = self.cursor.read_i32::<LittleEndian>()?;
                    export.serialization_before_serialization_dependencies =
                        self.cursor.read_i32::<LittleEndian>()?;
                    export.create_before_serialization_dependencies =
                        self.cursor.read_i32::<LittleEndian>()?;
                    export.serialization_before_create_dependencies =
                        self.cursor.read_i32::<LittleEndian>()?;
                    export.create_before_create_dependencies =
                        self.cursor.read_i32::<LittleEndian>()?;
                }

                self.exports.push(export.into());
            }
        }

        if self.depends_offset > 0 {
            let mut depends_map = Vec::with_capacity(self.export_count as usize);

            self.cursor
                .seek(SeekFrom::Start(self.depends_offset as u64))?;

            for _i in 0..self.export_count as usize {
                let size = self.cursor.read_i32::<LittleEndian>()?;
                let mut data: Vec<i32> = Vec::new();
                for _j in 0..size {
                    data.push(self.cursor.read_i32::<LittleEndian>()?);
                }
                depends_map.push(data);
            }
            self.depends_map = Some(depends_map);
        }

        if self.soft_package_reference_offset > 0 {
            let mut soft_package_reference_list =
                Vec::with_capacity(self.soft_package_reference_count as usize);

            self.cursor
                .seek(SeekFrom::Start(self.soft_package_reference_offset as u64))?;

            for _i in 0..self.soft_package_reference_count as usize {
                if let Some(reference) = self.cursor.read_string()? {
                    soft_package_reference_list.push(reference);
                }
            }
            self.soft_package_reference_list = Some(soft_package_reference_list);
        }

        // TODO: Asset registry data parsing should be here

        if self.world_tile_info_offset > 0 {
            self.cursor
                .seek(SeekFrom::Start(self.world_tile_info_offset as u64))?;
            self.world_tile_info = Some(FWorldTileInfo::new(self, self.engine_version)?);
        }

        if self.use_separate_bulk_data_files {
            self.cursor
                .seek(SeekFrom::Start(self.preload_dependency_offset as u64))?;
            let mut preload_dependencies = Vec::new();

            for _i in 0..self.preload_dependency_count as usize {
                preload_dependencies.push(self.cursor.read_i32::<LittleEndian>()?);
            }
            self.preload_dependencies = Some(preload_dependencies);
        }

        if self.header_offset > 0 && self.exports.len() > 0 {
            for i in 0..self.exports.len() {
                let unk_export = match &self.exports[i] {
                    Export::UnknownExport(export) => Some(export.clone()),
                    _ => None,
                };

                if let Some(unk_export) = unk_export {
                    let export: Result<Export, Error> = match self.read_export(&unk_export, i) {
                        Ok(e) => Ok(e),
                        Err(_e) => {
                            //todo: warning?
                            self.cursor
                                .seek(SeekFrom::Start(unk_export.serial_offset as u64))?;
                            Ok(RawExport::from_unk(unk_export.clone(), self)?.into())
                        }
                    };
                    self.exports[i] = export?;
                }
            }
        }

        // // ------------------------------------------------------------
        // header end, main data start
        // ------------------------------------------------------------

        /*println!(
            "data (len: {:?}): {:02X?}",
            self.cursor.get_ref().len(),
            self.cursor.get_ref()
        );*/

        Ok(())
    }

    fn read_export(&mut self, unk_export: &UnknownExport, i: usize) -> Result<Export, Error> {
        let next_starting = match i < (self.exports.len() - 1) {
            true => match &self.exports[i + 1] {
                Export::UnknownExport(next_export) => next_export.serial_offset as u64,
                _ => self.data_length - 4,
            },
            false => self.data_length - 4,
        };

        self.cursor
            .seek(SeekFrom::Start(unk_export.serial_offset as u64))?;

        //todo: manual skips
        let export_class_type = self.get_export_class_type(unk_export.class_index).ok_or(
            Error::invalid_package_index("Unknown class type".to_string()),
        )?;
        let mut export: Export = match export_class_type.content.as_str() {
            "Level" => LevelExport::from_unk(unk_export, self, next_starting)?.into(),
            "StringTable" => StringTableExport::from_unk(unk_export, self)?.into(),
            "Enum" | "UserDefinedEnum" => EnumExport::from_unk(unk_export, self)?.into(),
            "Function" => StructExport::from_unk(unk_export, self)?.into(),
            _ => {
                if export_class_type.content.ends_with("DataTable") {
                    DataTableExport::from_unk(unk_export, self)?.into()
                } else if export_class_type
                    .content
                    .ends_with("BlueprintGeneratedClass")
                {
                    let class_export = ClassExport::from_unk(unk_export, self)?;

                    for entry in &class_export.struct_export.loaded_properties {
                        if let FProperty::FMapProperty(map) = entry {
                            let key_override = match &*map.key_prop {
                                FProperty::FStructProperty(struct_property) => {
                                    match is_import(struct_property.struct_value.index) {
                                        true => self
                                            .get_import(struct_property.struct_value.index)
                                            .map(|e| e.object_name.content.to_owned()),
                                        false => None,
                                    }
                                }
                                _ => None,
                            };
                            if let Some(key) = key_override {
                                self.map_key_override
                                    .insert(map.generic_property.name.content.to_owned(), key);
                            }

                            let value_override = match &*map.key_prop {
                                FProperty::FStructProperty(struct_property) => {
                                    match is_import(struct_property.struct_value.index) {
                                        true => self
                                            .get_import(struct_property.struct_value.index)
                                            .map(|e| e.object_name.content.to_owned()),
                                        false => None,
                                    }
                                }
                                _ => None,
                            };

                            if let Some(value) = value_override {
                                self.map_value_override
                                    .insert(map.generic_property.name.content.to_owned(), value);
                            }
                        }
                    }
                    class_export.into()
                } else if export_class_type.content.ends_with("Property") {
                    PropertyExport::from_unk(unk_export, self)?.into()
                } else {
                    NormalExport::from_unk(unk_export, self)?.into()
                }
            }
        };

        let extras_len = next_starting as i64 - self.cursor.position() as i64;
        if extras_len < 0 {
            // todo: warning?

            self.cursor
                .seek(SeekFrom::Start(unk_export.serial_offset as u64))?;
            return Ok(RawExport::from_unk(unk_export.clone(), self)?.into());
        } else {
            if let Some(normal_export) = export.get_normal_export_mut() {
                let mut extras = vec![0u8; extras_len as usize];
                self.cursor.read_exact(&mut extras)?;
                normal_export.extras = extras;
            }
        }

        Ok(export)
    }

    fn write_header(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        name_offset: i32,
        import_offset: i32,
        export_offset: i32,
        depends_offset: i32,
        soft_package_reference_offset: i32,
        asset_registry_data_offset: i32,
        world_tile_info_offset: i32,
        preload_dependency_offset: i32,
        header_offset: i32,
        bulk_data_start_offset: i64,
    ) -> Result<(), Error> {
        cursor.write_u32::<BigEndian>(UE4_ASSET_MAGIC)?;
        cursor.write_i32::<LittleEndian>(self.legacy_file_version)?;

        if self.legacy_file_version != 4 {
            match self.unversioned {
                true => cursor.write_i32::<LittleEndian>(0)?,
                false => cursor.write_i32::<LittleEndian>(864)?,
            };
        }

        match self.unversioned {
            true => cursor.write_i32::<LittleEndian>(0)?,
            false => cursor.write_i32::<LittleEndian>(self.engine_version)?,
        };

        cursor.write_i32::<LittleEndian>(self.file_license_version)?;
        if self.legacy_file_version <= -2 {
            match self.unversioned {
                true => cursor.write_i32::<LittleEndian>(0)?,
                false => {
                    cursor.write_i32::<LittleEndian>(self.custom_version.len() as i32)?;
                    for custom_version in &self.custom_version {
                        cursor.write(&custom_version.guid)?;
                        cursor.write_i32::<LittleEndian>(custom_version.version)?;
                    }
                }
            };
        }

        cursor.write_i32::<LittleEndian>(header_offset)?;
        cursor.write_string(&Some(self.folder_name.clone()))?;
        cursor.write_u32::<LittleEndian>(self.package_flags)?;
        cursor.write_i32::<LittleEndian>(self.name_map_index_list.len() as i32)?;
        cursor.write_i32::<LittleEndian>(name_offset)?;

        if self.engine_version >= VER_UE4_SERIALIZE_TEXT_IN_PACKAGES {
            cursor.write_i32::<LittleEndian>(self.gatherable_text_data_count)?;
            cursor.write_i32::<LittleEndian>(self.gatherable_text_data_offset)?;
        }

        cursor.write_i32::<LittleEndian>(self.exports.len() as i32)?;
        cursor.write_i32::<LittleEndian>(export_offset)?;
        cursor.write_i32::<LittleEndian>(self.imports.len() as i32)?;
        cursor.write_i32::<LittleEndian>(import_offset)?;
        cursor.write_i32::<LittleEndian>(depends_offset)?;

        if self.engine_version >= VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP {
            cursor.write_i32::<LittleEndian>(self.soft_package_reference_count)?;
            cursor.write_i32::<LittleEndian>(soft_package_reference_offset)?;
        }

        if self.engine_version >= VER_UE4_ADDED_SEARCHABLE_NAMES {
            cursor.write_i32::<LittleEndian>(self.searchable_names_offset)?;
        }

        cursor.write_i32::<LittleEndian>(self.thumbnail_table_offset)?;
        cursor.write(&self.package_guid)?;
        cursor.write_i32::<LittleEndian>(self.generations.len() as i32)?;

        for _ in 0..self.generations.len() {
            cursor.write_i32::<LittleEndian>(self.exports.len() as i32)?;
            cursor.write_i32::<LittleEndian>(self.name_map_index_list.len() as i32)?;
        }

        if self.engine_version >= VER_UE4_ENGINE_VERSION_OBJECT {
            self.engine_version_recorded.write(cursor)?;
        } else {
            cursor.write_u32::<LittleEndian>(self.engine_version_recorded.build)?;
        }

        if self.engine_version >= VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION {
            self.engine_version_recorded.write(cursor)?;
        }

        cursor.write_u32::<LittleEndian>(self.compression_flags)?;
        cursor.write_i32::<LittleEndian>(0)?; // numCompressedChunks
        cursor.write_u32::<LittleEndian>(self.package_source)?;
        cursor.write_i32::<LittleEndian>(0)?; // numAdditionalPackagesToCook

        if self.legacy_file_version > -7 {
            cursor.write_i32::<LittleEndian>(0)?; // numTextureallocations
        }

        cursor.write_i32::<LittleEndian>(asset_registry_data_offset)?;
        cursor.write_i64::<LittleEndian>(bulk_data_start_offset)?;

        if self.engine_version >= VER_UE4_WORLD_LEVEL_INFO {
            cursor.write_i32::<LittleEndian>(world_tile_info_offset)?;
        }

        if self.engine_version >= VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS {
            cursor.write_i32::<LittleEndian>(self.chunk_ids.len() as i32)?;
            for chunk_id in &self.chunk_ids {
                cursor.write_i32::<LittleEndian>(*chunk_id)?;
            }
        } else if self.engine_version >= VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE {
            cursor.write_i32::<LittleEndian>(self.chunk_ids[0])?;
        }

        if self.engine_version >= VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
            cursor.write_i32::<LittleEndian>(
                self.preload_dependencies
                    .as_ref()
                    .map(|e| e.len())
                    .unwrap_or(0) as i32,
            )?;
            cursor.write_i32::<LittleEndian>(preload_dependency_offset)?;
        }

        Ok(())
    }

    fn write_export_header(
        &self,
        unk: &UnknownExport,
        cursor: &mut Cursor<Vec<u8>>,
        serial_size: i64,
        serial_offset: i64,
    ) -> Result<(), Error> {
        cursor.write_i32::<LittleEndian>(unk.class_index)?;
        cursor.write_i32::<LittleEndian>(unk.super_index)?;

        if self.engine_version >= VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS {
            cursor.write_i32::<LittleEndian>(unk.template_index)?;
        }

        cursor.write_i32::<LittleEndian>(unk.outer_index)?;
        self.write_fname(cursor, &unk.object_name)?;
        cursor.write_u32::<LittleEndian>(unk.object_flags)?;

        if self.engine_version < VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
            cursor.write_i32::<LittleEndian>(serial_size as i32)?;
            cursor.write_i32::<LittleEndian>(serial_offset as i32)?;
        } else {
            cursor.write_i64::<LittleEndian>(serial_size)?;
            cursor.write_i64::<LittleEndian>(serial_offset)?;
        }

        cursor.write_i32::<LittleEndian>(match unk.forced_export {
            true => 1,
            false => 0,
        })?;
        cursor.write_i32::<LittleEndian>(match unk.not_for_client {
            true => 1,
            false => 0,
        })?;
        cursor.write_i32::<LittleEndian>(match unk.not_for_server {
            true => 1,
            false => 0,
        })?;
        cursor.write(&unk.package_guid)?;
        cursor.write_u32::<LittleEndian>(unk.package_flags)?;

        if self.engine_version >= VER_UE4_LOAD_FOR_EDITOR_GAME {
            cursor.write_i32::<LittleEndian>(match unk.not_always_loaded_for_editor_game {
                true => 1,
                false => 0,
            })?;
        }

        if self.engine_version >= VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
            cursor.write_i32::<LittleEndian>(match unk.is_asset {
                true => 1,
                false => 0,
            })?;
        }

        if self.engine_version >= VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
            cursor.write_i32::<LittleEndian>(unk.first_export_dependency)?;
            cursor
                .write_i32::<LittleEndian>(unk.serialization_before_serialization_dependencies)?;
            cursor.write_i32::<LittleEndian>(unk.create_before_serialization_dependencies)?;
            cursor.write_i32::<LittleEndian>(unk.serialization_before_create_dependencies)?;
            cursor.write_i32::<LittleEndian>(unk.create_before_create_dependencies)?;
        }
        Ok(())
    }

    pub fn write_data(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        uexp_cursor: Option<&mut Cursor<Vec<u8>>>,
    ) -> Result<(), Error> {
        if self.use_separate_bulk_data_files != uexp_cursor.is_some() {
            return Err(Error::no_data(format!(
                "use_separate_bulk_data_files is {} but uexp_cursor is {}",
                self.use_separate_bulk_data_files,
                match uexp_cursor.is_some() {
                    true => "Some(...)",
                    false => "None",
                }
            )));
        }
        self.write_header(
            cursor,
            self.name_offset,
            self.import_offset,
            self.export_offset,
            self.depends_offset,
            self.soft_package_reference_offset,
            self.asset_registry_data_offset,
            self.world_tile_info_offset,
            self.preload_dependency_offset,
            self.header_offset,
            self.bulk_data_start_offset,
        )?;

        let name_offset = match self.name_map_index_list.len() != 0 {
            true => cursor.position() as i32,
            false => 0,
        };

        for name in &self.name_map_index_list {
            cursor.write_string(&Some(name.clone()))?;

            if self.engine_version >= VER_UE4_NAME_HASHES_SERIALIZED {
                match self.override_name_map_hashes.get(name) {
                    Some(e) => cursor.write_u32::<LittleEndian>(*e)?,
                    None => cursor.write_u32::<LittleEndian>(crc::generate_hash(name))?,
                };
            }
        }

        let import_offset = match self.imports.len() != 0 {
            true => cursor.position() as i32,
            false => 0,
        };

        for import in &self.imports {
            self.write_fname(cursor, &import.class_package)?;
            self.write_fname(cursor, &import.class_name)?;
            cursor.write_i32::<LittleEndian>(import.outer_index)?;
            self.write_fname(cursor, &import.object_name)?;
        }

        let export_offset = match self.exports.len() != 0 {
            true => cursor.position() as i32,
            false => 0,
        };

        for export in &self.exports {
            let unk: &UnknownExport = export.get_unknown_export();
            self.write_export_header(unk, cursor, unk.serial_size, unk.serial_offset)?;
        }

        let depends_offset = match self.depends_map {
            Some(_) => cursor.position() as i32,
            None => 0,
        };

        if let Some(ref map) = self.depends_map {
            for i in 0..self.exports.len() {
                let dummy = Vec::new();
                let current_data = match map.get(i) {
                    Some(e) => e,
                    None => &dummy,
                };
                cursor.write_i32::<LittleEndian>(current_data.len() as i32)?;
                for i in current_data {
                    cursor.write_i32::<LittleEndian>(*i)?;
                }
            }
        }

        let soft_package_reference_offset = match self.soft_package_reference_list {
            Some(_) => cursor.position() as i32,
            None => 0,
        };

        if let Some(ref package_references) = self.soft_package_reference_list {
            for reference in package_references {
                cursor.write_string(&Some(reference.clone()))?;
            }
        }

        // todo: asset registry data support
        let asset_registry_data_offset = match self.asset_registry_data_offset != 0 {
            true => cursor.position() as i32,
            false => 0,
        };
        if self.asset_registry_data_offset != 0 {
            cursor.write_i32::<LittleEndian>(0)?; // asset registry data length
        }

        let world_tile_info_offset = match self.world_tile_info {
            Some(_) => cursor.position() as i32,
            None => 0,
        };

        if let Some(ref world_tile_info) = self.world_tile_info {
            world_tile_info.write(self, cursor)?;
        }

        let preload_dependency_offset = match self.use_separate_bulk_data_files {
            true => cursor.position() as i32,
            false => 0,
        };

        if self.use_separate_bulk_data_files {
            for entry in self.preload_dependencies.as_ref().ok_or(Error::no_data(
                "use_separate_bulk_data_files: true but no preload_dependencies found".to_string(),
            ))? {
                cursor.write_i32::<LittleEndian>(*entry)?;
            }
        }

        let header_offset = match self.exports.len() != 0 {
            true => cursor.position() as i32,
            false => 0,
        };

        let mut category_starts = Vec::with_capacity(self.exports.len());

        let final_cursor_pos = cursor.position();
        let bulk_cursor = match self.use_separate_bulk_data_files {
            true => uexp_cursor.unwrap(),
            false => cursor,
        };

        for export in &self.exports {
            category_starts.push(match self.use_separate_bulk_data_files {
                true => bulk_cursor.position() + final_cursor_pos,
                false => bulk_cursor.position(),
            });
            export.write(self, bulk_cursor)?;
            if let Some(normal_export) = export.get_normal_export() {
                bulk_cursor.write(&normal_export.extras)?;
            }
        }
        bulk_cursor.write(&[0xc1, 0x83, 0x2a, 0x9e])?;

        let bulk_data_start_offset = match self.exports.len() != 0 {
            true => match self.use_separate_bulk_data_files {
                true => final_cursor_pos as i64 + bulk_cursor.position() as i64,
                false => cursor.position() as i64,
            },
            false => 0,
        };

        if self.exports.len() > 0 {
            cursor.seek(SeekFrom::Start(export_offset as u64))?;
            for i in 0..self.exports.len() {
                let unk = &self.exports[i].get_unknown_export();
                let next_loc = match self.exports.len() - 1 > i {
                    true => category_starts[i + 1] as i64,
                    false => self.bulk_data_start_offset,
                };
                self.write_export_header(
                    unk,
                    cursor,
                    next_loc - category_starts[i] as i64,
                    category_starts[i] as i64,
                )?;
            }
        }

        cursor.seek(SeekFrom::Start(0))?;
        self.write_header(
            cursor,
            name_offset,
            import_offset,
            export_offset,
            depends_offset,
            soft_package_reference_offset,
            asset_registry_data_offset,
            world_tile_info_offset,
            preload_dependency_offset,
            header_offset,
            bulk_data_start_offset,
        )?;

        cursor.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}

// custom debug implementation to not print the whole data buffer
impl Debug for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Asset")
            .field("data_len", &self.cursor.get_ref().len())
            .field("info", &self.info)
            .field(
                "use_separate_bulk_data_files",
                &self.use_separate_bulk_data_files,
            )
            .field("engine_version", &self.engine_version)
            .field("legacy_file_version", &self.legacy_file_version)
            .field("unversioned", &self.unversioned)
            .field("file_license_version", &self.file_license_version)
            .field("custom_version", &self.custom_version)
            // imports
            // exports
            // depends map
            // soft package reference list
            // asset registry data
            // world tile info
            // preload dependencies
            .field("generations", &self.generations)
            .field("package_guid", &self.package_guid)
            .field("engine_version_recorded", &self.engine_version_recorded)
            .field("engine_version_compatible", &self.engine_version_compatible)
            .field("chunk_ids", &self.chunk_ids)
            .field("package_flags", &self.package_flags)
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
            .field("export_count", &self.export_count)
            .field("export_offset", &self.export_offset)
            .field("import_count", &self.import_count)
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

pub fn is_import(index: i32) -> bool {
    return index < 0;
}

pub fn is_export(index: i32) -> bool {
    return index > 0;
}

#[derive(Debug, Clone)]
pub struct EngineVersion {
    major: u16,
    minor: u16,
    patch: u16,
    build: u32,
    branch: Option<String>,
}
impl EngineVersion {
    fn new(major: u16, minor: u16, patch: u16, build: u32, branch: Option<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
            branch,
        }
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let major = cursor.read_u16::<LittleEndian>()?;
        let minor = cursor.read_u16::<LittleEndian>()?;
        let patch = cursor.read_u16::<LittleEndian>()?;
        let build = cursor.read_u32::<LittleEndian>()?;
        let branch = cursor.read_string()?;

        Ok(Self::new(major, minor, patch, build, branch))
    }

    fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        cursor.write_u16::<LittleEndian>(self.major)?;
        cursor.write_u16::<LittleEndian>(self.minor)?;
        cursor.write_u16::<LittleEndian>(self.patch)?;
        cursor.write_u32::<LittleEndian>(self.build)?;
        cursor.write_string(&self.branch)?;
        Ok(())
    }

    fn unknown() -> Self {
        Self::new(0, 0, 0, 0, None)
    }
}
