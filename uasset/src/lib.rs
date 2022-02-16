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
    use std::fmt::{Debug, Formatter};
    use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

    use byteorder::{ReadBytesExt, LittleEndian, BigEndian};

    pub mod ue4version;

    pub type Guid = [u8; 16];
    #[derive(Debug)]
    pub struct CustomVersion {
        guid: Guid,
        version: i32,
    }
    #[derive(Debug)]
    pub struct GenerationInfo {
        export_count: i32,
        name_count: i32,
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
        hashes: u32
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
            self.folder_name = read_string(&mut self.cursor)?;

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

        pub fn parse_data(&mut self) -> Result<(), Error> {
            println!("Parsing data...");

            self.parse_header()?;
            // self.cursor.seek(SeekFrom::Start(self.name_offset as u64))?;

            // for i in 0..self.name_count {
            //     let s = read_string(&mut self.cursor)?;

            //     if self.engine_version >= ue4version::VER_UE4_NAME_HASHES_SERIALIZED && !s.is_empty() {
            //         self.hashes = self.cursor.
            //     }
            // }
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

    // read string of format <length u32><string><null>
    fn read_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, Error> {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        let mut len = u32::from_le_bytes(buf);

        if len == 0 {
            return Ok(String::new());
        }

        let mut buf = vec![0u8; len as usize - 1];
        cursor.read_exact(&mut buf)?;
        cursor.seek(SeekFrom::Current(1))?;
        Ok(String::from_utf8(buf).unwrap_or(String::from("None")))
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
            let branch = read_string(cursor)?;

            Ok(Self::new(major, minor, patch, build, branch))
        }

        fn unknown() -> Self {
            Self::new(0, 0, 0, 0, String::from(""))
        }
    }
}
