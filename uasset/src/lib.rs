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
    use std::fmt::{Debug, Formatter};
    use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

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
            }
        }

        pub fn parse_data(&mut self) -> Result<(), Error> {
            println!("Parsing data...");

            // reuseable buffers for reading
            let mut buf4 = [0u8; 4];
            let mut buf8 = [0u8; 8];

            // seek to start
            self.cursor.seek(SeekFrom::Start(0))?;

            // read and check magic
            self.cursor.read_exact(&mut buf4)?;
            if u32::from_be_bytes(buf4) != UE4_ASSET_MAGIC {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "File is not a valid uasset file",
                ));
            }

            // read legacy version
            self.cursor.read_exact(&mut buf4)?;
            self.legacy_file_version = i32::from_le_bytes(buf4);
            println!("Legacy file version: {}", self.legacy_file_version);
            if self.legacy_file_version != -4 {
                // LegacyUE3Version for backwards-compatibility with UE3 games: always 864 in versioned assets, always 0 in unversioned assets
                self.cursor.read_exact(&mut buf4)?;
            }

            // read unreal version
            self.cursor.read_exact(&mut buf4)?;
            let file_version = i32::from_le_bytes(buf4);
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
            self.cursor.read_exact(&mut buf4)?;
            self.file_license_version = i32::from_le_bytes(buf4);

            // read custom versions container
            if self.legacy_file_version <= -2 {
                // TODO: support for enum-based custom versions

                // read custom version count
                self.cursor.read_exact(&mut buf4)?;
                let custom_versions_count = i32::from_le_bytes(buf4);

                for _ in 0..custom_versions_count {
                    // read guid
                    let mut guid = [0u8; 16];
                    self.cursor.read_exact(&mut guid)?;
                    // read version
                    self.cursor.read_exact(&mut buf4)?;
                    let version = i32::from_le_bytes(buf4);

                    self.custom_version.push(CustomVersion { guid, version });
                }
            }

            // read header offset
            self.cursor.read_exact(&mut buf4)?;
            self.header_offset = i32::from_le_bytes(buf4);

            // read folder name
            self.folder_name = read_string(&mut self.cursor)?;

            // read package flags
            self.cursor.read_exact(&mut buf4)?;
            self.package_flags = u32::from_le_bytes(buf4);

            // read name count and offset
            self.cursor.read_exact(&mut buf4)?;
            self.name_count = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            self.name_offset = i32::from_le_bytes(buf4);
            // read text gatherable data
            if self.engine_version >= ue4version::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES {
                self.cursor.read_exact(&mut buf4)?;
                self.gatherable_text_data_count = i32::from_le_bytes(buf4);
                self.cursor.read_exact(&mut buf4)?;
                self.gatherable_text_data_offset = i32::from_le_bytes(buf4);
            }

            // read count and offset for exports, imports, depends, soft package references, searchable names, thumbnail table
            self.cursor.read_exact(&mut buf4)?;
            self.export_count = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            self.export_offset = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            self.import_count = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            self.import_offset = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            self.depends_offset = i32::from_le_bytes(buf4);
            if self.engine_version >= ue4version::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP {
                self.cursor.read_exact(&mut buf4)?;
                self.soft_package_reference_count = i32::from_le_bytes(buf4);
                self.cursor.read_exact(&mut buf4)?;
                self.soft_package_reference_offset = i32::from_le_bytes(buf4);
            }
            if self.engine_version >= ue4version::VER_UE4_ADDED_SEARCHABLE_NAMES {
                self.cursor.read_exact(&mut buf4)?;
                self.searchable_names_offset = i32::from_le_bytes(buf4);
            }
            self.cursor.read_exact(&mut buf4)?;
            self.thumbnail_table_offset = i32::from_le_bytes(buf4);

            println!("Header offset: {}", self.header_offset);

            // read guid
            self.cursor.read_exact(&mut self.package_guid)?;

            println!("Package GUID: {:02X?}", self.package_guid);

            // raed generations
            self.cursor.read_exact(&mut buf4)?;
            let generations_count = i32::from_le_bytes(buf4);
            for _ in 0..generations_count {
                self.cursor.read_exact(&mut buf4)?;
                let export_count = i32::from_le_bytes(buf4);
                self.cursor.read_exact(&mut buf4)?;
                let name_count = i32::from_le_bytes(buf4);
                self.generations.push(GenerationInfo {
                    export_count,
                    name_count,
                });
            }

            // read advanced engine version
            if self.engine_version >= ue4version::VER_UE4_ENGINE_VERSION_OBJECT {
                self.engine_version_recorded = EngineVersion::read(&mut self.cursor)?;
            } else {
                self.cursor.read_exact(&mut buf4)?;
                self.engine_version_recorded =
                    EngineVersion::new(4, 0, 0, u32::from_le_bytes(buf4), String::from(""));
            }
            if self.engine_version
                >= ue4version::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
            {
                self.engine_version_compatible = EngineVersion::read(&mut self.cursor)?;
            } else {
                self.engine_version_compatible = self.engine_version_recorded.clone();
            }

            // read compression data
            self.cursor.read_exact(&mut buf4)?;
            self.compression_flags = u32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf4)?;
            let compression_block_count = u32::from_le_bytes(buf4);
            if compression_block_count > 0 {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Compression block count is not zero",
                ));
            }

            self.cursor.read_exact(&mut buf4)?;
            self.package_source = u32::from_le_bytes(buf4);

            // some other old unsupported stuff
            self.cursor.read_exact(&mut buf4)?;
            let additional_to_cook = i32::from_le_bytes(buf4);
            if additional_to_cook != 0 {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Additional to cook is not zero",
                ));
            }
            if self.legacy_file_version > -7 {
                self.cursor.read_exact(&mut buf4)?;
                let texture_allocations_count = i32::from_le_bytes(buf4);
                if texture_allocations_count != 0 {
                    return Err(Error::new(
                        ErrorKind::Unsupported,
                        "Texture allocations count is not zero",
                    ));
                }
            }

            self.cursor.read_exact(&mut buf4)?;
            self.asset_registry_data_offset = i32::from_le_bytes(buf4);
            self.cursor.read_exact(&mut buf8)?;
            self.bulk_data_start_offset = i64::from_le_bytes(buf8);

            if self.engine_version >= ue4version::VER_UE4_WORLD_LEVEL_INFO {
                self.cursor.read_exact(&mut buf4)?;
                self.world_tile_info_offset = i32::from_le_bytes(buf4);
            }

            if self.engine_version >= ue4version::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS
            {
                self.cursor.read_exact(&mut buf4)?;
                let chunk_id_count = i32::from_le_bytes(buf4);

                for _ in 0..chunk_id_count {
                    self.cursor.read_exact(&mut buf4)?;
                    let chunk_id = i32::from_le_bytes(buf4);
                    self.chunk_ids.push(chunk_id);
                }
            } else if self.engine_version
                >= ue4version::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE
            {
                self.chunk_ids = vec![];
                self.cursor.read_exact(&mut buf4)?;
                self.chunk_ids[0] = i32::from_le_bytes(buf4);
            }

            if self.engine_version >= ue4version::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
                self.cursor.read_exact(&mut buf4)?;
                self.preload_dependency_count = i32::from_le_bytes(buf4);
                self.cursor.read_exact(&mut buf4)?;
                self.preload_dependency_offset = i32::from_le_bytes(buf4);
            }

            // ------------------------------------------------------------
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
            cursor.read_exact(&mut buf2)?;
            let major = u16::from_le_bytes(buf2);
            cursor.read_exact(&mut buf2)?;
            let minor = u16::from_le_bytes(buf2);
            cursor.read_exact(&mut buf2)?;
            let patch = u16::from_le_bytes(buf2);
            let mut buf4 = [0u8; 4];
            cursor.read_exact(&mut buf4)?;
            let build = u32::from_le_bytes(buf4);
            let branch = read_string(cursor)?;

            Ok(Self::new(major, minor, patch, build, branch))
        }

        fn unknown() -> Self {
            Self::new(0, 0, 0, 0, String::from(""))
        }
    }
}
