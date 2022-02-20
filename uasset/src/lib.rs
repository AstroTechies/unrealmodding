/*#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}*/

//pub use crate::ue4version;

pub mod uasset {
    use std::collections::HashMap;
    use std::collections::hash_map::DefaultHasher;
    use std::fmt::{Debug, Formatter};
    use std::hash::{Hash, Hasher};
    use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

    use byteorder::{ReadBytesExt, LittleEndian, BigEndian};

    use crate::uasset::ue4version::{VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS, VER_UE4_64BIT_EXPORTMAP_SERIALSIZES, VER_UE4_LOAD_FOR_EDITOR_GAME, VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT, VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS};

    use self::cursor_ext::CursorExt;
    use self::flags::{EPackageFlags, EObjectFlags};
    use self::structs::world_tile::FWorldTileInfo;
    use self::unreal_types::Guid;

    pub mod flags;
    pub mod ue4version;
    pub mod types;
    pub mod unreal_types;
    pub mod cursor_ext;
    pub mod structs;
    pub mod properties;
    pub mod exports;
    use unreal_types::{FName, CustomVersion, GenerationInfo};

    #[derive(Debug)]
    pub struct Import {
        class_package: FName,
        class_name: FName,
        outer_index: i32,
        object_name: FName
    }

    impl Import {
        pub fn new(class_package: FName, class_name: FName, outer_index: i32, object_name: FName) -> Self {
            Import { class_package, class_name, object_name, outer_index }
        }
    }



    const UE4_ASSET_MAGIC: u32 = u32::from_be_bytes([0xc1, 0x83, 0x2a, 0x9e]);

    //#[derive(Debug)]
    pub struct Asset {
        // raw data
        cursor: Cursor<Vec<u8>>,

        // parsed data
        pub info: String,
        pub use_seperate_bulk_data_files: bool,
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
        imports: Vec<Import>,
        exports: Vec<Export>,
        depends_map: Option<Vec<Vec<i32>>>,
        soft_package_reference_list: Option<Vec<String>>,
        world_tile_info: Option<FWorldTileInfo>,
        preload_dependencies: Option<Vec<i32>>
    }

    impl Asset {
        pub fn new(raw_data: Vec<u8>) -> Self {
            Asset {
                cursor: Cursor::new(raw_data),

                info: String::from("Serialized with unrealmodding/uasset"),
                use_seperate_bulk_data_files: false,
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
                hashes: 0
            }
        }

        fn parse_header(&mut self) -> Result<(), Error> {
            // reuseable buffers for reading

            // seek to start
            self.cursor.seek(SeekFrom::Start(0))?;

            // read and check magic
            if self.cursor.read_u32::<BigEndian>()? != UE4_ASSET_MAGIC {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "File is not a valid uasset file",
                ));
            }

            // read legacy version
            self.legacy_file_version = self.cursor.read_i32::<LittleEndian>()?;
            println!("Legacy file version: {}", self.legacy_file_version);
            if self.legacy_file_version != -4 {
                // LegacyUE3Version for backwards-compatibility with UE3 games: always 864 in versioned assets, always 0 in unversioned assets
                self.cursor.read_exact(&mut [0u8; 4])?;
            }

            // read unreal version
            let file_version = self.cursor.read_i32::<LittleEndian>()?;
            println!("File version: {}", file_version);

            self.unversioned = file_version == ue4version::UNKNOWN;
            println!("Unversioned: {}", self.unversioned);

            if self.unversioned {
                if self.engine_version == ue4version::UNKNOWN {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Cannot begin serialization of an unversioned asset before an engine version is manually specified",
                    ));
                }
            } else {
                self.engine_version = file_version;
            }

            println!("Engine version: {}", self.engine_version);

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

                    self.custom_version.push(CustomVersion { guid, version });
                }
            }

            // read header offset
            self.header_offset = self.cursor.read_i32::<LittleEndian>()?;

            // read folder name
            self.folder_name = self.cursor.read_string()?;

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

            println!("Header offset: {}", self.header_offset);

            // read guid
            self.cursor.read_exact(&mut self.package_guid)?;

            println!("Package GUID: {:02X?}", self.package_guid);

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
                    EngineVersion::new(4, 0, 0, self.cursor.read_u32::<LittleEndian>()?, String::from(""));
            }
            if self.engine_version
                >= ue4version::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
            {
                self.engine_version_compatible = EngineVersion::read(&mut self.cursor)?;
            } else {
                self.engine_version_compatible = self.engine_version_recorded.clone();
            }

            // read compression data
            self.compression_flags = self.cursor.read_u32::<LittleEndian>()?;
            let compression_block_count = self.cursor.read_u32::<LittleEndian>()?;
            if compression_block_count > 0 {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Compression block count is not zero",
                ));
            }

            self.package_source = self.cursor.read_u32::<LittleEndian>()?;

            // some other old unsupported stuff
            let additional_to_cook = self.cursor.read_i32::<LittleEndian>()?;
            if additional_to_cook != 0 {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Additional to cook is not zero",
                ));
            }
            if self.legacy_file_version > -7 {
                let texture_allocations_count = self.cursor.read_i32::<LittleEndian>()?;
                if texture_allocations_count != 0 {
                    return Err(Error::new(
                        ErrorKind::Unsupported,
                        "Texture allocations count is not zero",
                    ));
                }
            }

            self.asset_registry_data_offset = self.cursor.read_i32::<LittleEndian>()?;
            self.bulk_data_start_offset = self.cursor.read_i64::<LittleEndian>()?;

            if self.engine_version >= ue4version::VER_UE4_WORLD_LEVEL_INFO {
                self.world_tile_info_offset = self.cursor.read_i32::<LittleEndian>()?;
            }

            if self.engine_version >= ue4version::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS
            {
                let chunk_id_count = self.cursor.read_i32::<LittleEndian>()?;

                for _ in 0..chunk_id_count {
                    let chunk_id = self.cursor.read_i32::<LittleEndian>()?;
                    self.chunk_ids.push(chunk_id);
                }
            } else if self.engine_version
                >= ue4version::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE
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

        fn read_name_map_string(&mut self) -> Result<(u32, String), Error> {
            let s = self.cursor.read_string()?;
            let mut hashes = 0;
            if self.engine_version >= ue4version::VER_UE4_NAME_HASHES_SERIALIZED && !s.is_empty() {
                hashes = self.cursor.read_u32::<LittleEndian>()?;
            }
            Ok((hashes, s))
        }

        fn fix_name_map_lookup(&mut self) {
            if self.name_map_index_list.len() > 0 && self.name_map_lookup.is_empty() {
                for i in 0..self.name_map_index_list.len() {
                    let mut s = DefaultHasher::new();
                    self.name_map_index_list[i].hash(&mut s);
                    self.name_map_lookup.insert(s.finish(), i as i32);
                }
            }
        }

        fn search_name_reference(&mut self, name: String) -> Option<i32> {
            self.fix_name_map_lookup();
            
            let mut s = DefaultHasher::new();
            name.hash(&mut s);
            
            match self.name_map_lookup.get(&s.finish()) {
                Some(e) => Some(*e),
                None => None
            }
        }

        fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
            self.fix_name_map_lookup();

            if !force_add_duplicates {
                let existing = self.search_name_reference(name);
                if existing.is_some() {
                    return existing.unwrap();
                }
            }

            let mut s = DefaultHasher::new();
            name.hash(&mut s);

            self.name_map_index_list.push(name);
            self.name_map_lookup.insert(s.finish(), self.name_map_lookup.len() as i32);
            (self.name_map_lookup.len() - 1) as i32
        }

        fn get_name_reference(&mut self, index: i32) -> String {
            self.fix_name_map_lookup();
            if index < 0 {
                return (-index).to_string(); // is this right even?
            }
            if index > self.name_map_index_list.len() as i32 {
                return index.to_string();
            }
            self.name_map_index_list[index as usize]
        }

        fn read_fname(&mut self) -> Result<FName, Error> {
            let name_map_pointer = self.cursor.read_i32::<LittleEndian>()?;
            let number = self.cursor.read_i32::<LittleEndian>()?;

            Ok(FName::new(self.get_name_reference(number), number))
        }

        fn get_import(self, index: i32) -> Option<Import> {
            if !is_import(index) {
                return None;
            }

            let index = -index - 1;
            if index < 0 || index > self.imports.len() as i32 {
                return None;
            }

            Some(self.imports[index as usize])
        }

        fn get_export(self, index: i32) -> Option<Export> {
            if !is_export(index) {
                return None;
            }

            let index = index - 1;

            if index < 0 || index >= self.exports.len() as i32 {
                return None;
            }

            Some(self.exports[index as usize])
        }
        
        pub fn parse_data(&mut self) -> Result<(), Error> {
            println!("Parsing data...");

            self.parse_header()?;
            self.cursor.seek(SeekFrom::Start(self.name_offset as u64))?;

            for i in 0..self.name_count {
                let name_map = self.read_name_map_string()?;
                if name_map.0 == 0 {
                    if let Some(entry) = self.override_name_map_hashes.get_mut(&name_map.1) {
                        *entry = 0u32;
                    }
                }
                self.add_name_reference(name_map.1, true);
            }

            if self.import_offset > 0 {
                self.cursor.seek(SeekFrom::Start(self.import_offset as u64));
                for i in 0..self.import_count {
                    self.imports.push(Import::new(self.read_fname()?, self.read_fname()?, self.cursor.read_i32::<LittleEndian>()?, self.read_fname()?));
                }
            }

            if self.export_offset > 0 {
                self.cursor.seek(SeekFrom::Start(self.export_offset as u64));
                for i in 0..self.export_count {
                    let mut export = Export::default();
                    export.class_index = self.cursor.read_i32::<LittleEndian>()?;
                    export.super_index = self.cursor.read_i32::<LittleEndian>()?;

                    if(self.engine_version >= VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS) {
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
                    self.cursor.read_exact(&mut self.package_guid)?;
                    export.package_flags = self.cursor.read_u32::<LittleEndian>()?;

                    if self.engine_version >= VER_UE4_LOAD_FOR_EDITOR_GAME {
                        export.not_always_loaded_for_editor_game = self.cursor.read_i32::<LittleEndian>()? == 1;
                    }

                    if self.engine_version >= VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
                        export.is_asset = self.cursor.read_i32::<LittleEndian>()? == 1;
                    }

                    if self.engine_version >= VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
                        export.first_export_dependency = self.cursor.read_i32::<LittleEndian>()?;
                        export.serialization_before_serialization_dependencies = self.cursor.read_i32::<LittleEndian>()?;
                        export.create_before_serialization_dependencies = self.cursor.read_i32::<LittleEndian>()?;
                        export.serialization_before_create_dependencies = self.cursor.read_i32::<LittleEndian>()?;
                        export.create_before_create_dependencies = self.cursor.read_i32::<LittleEndian>()?;
                    }

                    self.exports.push(export);
                }
            }

            if self.depends_offset > 0 {
                self.depends_map = Some(Vec::new());

                self.cursor.seek(SeekFrom::Start(self.depends_offset as u64))?;

                for i in 0..self.export_count {
                    let size = self.cursor.read_i32::<LittleEndian>()?;
                    let data: Vec<i32> = Vec::new();
                    for j in 0..size {
                        data.push(self.cursor.read_i32::<LittleEndian>()?);
                    } 
                    self.depends_map.unwrap().push(data);
                }
            } 

            if self.soft_package_reference_offset > 0 {
                self.soft_package_reference_list = Some(Vec::new());

                self.cursor.seek(SeekFrom::Start(self.soft_package_reference_offset as u64))?;
                
                for i in 0..self.soft_package_reference_count {
                    self.soft_package_reference_list.unwrap().push(self.cursor.read_string()?);
                }
            }

            // TODO: Asset registry data parsing should be here

            if self.world_tile_info_offset > 0 {
                self.cursor.seek(SeekFrom::Start(self.world_tile_info_offset as u64))?;
                self.world_tile_info = Some(FWorldTileInfo::new(&mut self.cursor, self.engine_version)?);
            }

            if self.use_seperate_bulk_data_files {
                self.cursor.seek(SeekFrom::Start(self.preload_dependency_offset as u64))?;
                self.preload_dependencies = Some(Vec::new());

                for i in 0..self.preload_dependency_count {
                    self.preload_dependencies.unwrap().push(self.cursor.read_i32::<LittleEndian>()?);
                }
            }

            if self.header_offset > 0 && self.exports.len() > 0 {
                for mut export in self.exports {
                    self.cursor.seek(SeekFrom::Start(export.serial_offset as u64))?;

                    //todo: implement skips

                    //is nextstarting if needed?
                    let next_starting = export.serial_offset;

                    let export_class_type_name = match is_import(export.class_index) {
                        true => self.get_import(export.class_index).map(|e| e.object_name).ok_or(Error::new(ErrorKind::Other, "Import not found"))?,
                        false => FName::new(export.class_index.to_string(), 0)
                    };

                    let export_class_type = export_class_type_name.content;
                    match export_class_type.as_str() {
                        "Level" => {

                        },
                        _ => {

                        }
                    };

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

            println!("Cursor offset: {:02X?}", self.cursor.position());

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
                    "use_seperate_bulk_data_files",
                    &self.use_seperate_bulk_data_files,
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

    fn is_import(index: i32) -> bool {
        return index < 0;
    }

    fn is_export(index: i32) -> bool {
        return index > 0;
    }

    #[derive(Debug, Clone)]
    pub struct EngineVersion {
        major: u16,
        minor: u16,
        patch: u16,
        build: u32,
        branch: String,
    }
    impl EngineVersion {
        fn new(major: u16, minor: u16, patch: u16, build: u32, branch: String) -> Self {
            Self {
                major,
                minor,
                patch,
                build,
                branch,
            }
        }

        fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
            let mut buf2 = [0u8; 2];
            let major = cursor.read_u16::<LittleEndian>()?;
            let minor = cursor.read_u16::<LittleEndian>()?;
            let patch = cursor.read_u16::<LittleEndian>()?;
            let mut buf4 = [0u8; 4];
            let build = cursor.read_u32::<LittleEndian>()?;
            let branch = cursor.read_string()?;

            Ok(Self::new(major, minor, patch, build, branch))
        }

        fn unknown() -> Self {
            Self::new(0, 0, 0, 0, String::from(""))
        }
    }
}
