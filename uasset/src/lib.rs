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
    use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

    pub mod ue4version;

    const UE4_ASSET_MAGIC: u32 = u32::from_be_bytes([0xc1, 0x83, 0x2a, 0x9e]);

    #[derive(Debug)]
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
        // custom version
        // imports
        // exports
        // depends map
        // soft package reference list
        // asset registry data
        // world tile info
        // preload dependencies
        // generations
        // package guid
        pub engine_version_recorded: i32,
        pub engine_version_compatible: i32,
        chunk_ids: Vec<i32>,
        // package flags
        pub package_source: u32,
        pub folder_name: Option<String>,
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
        bulk_data_start_offset: i32,
        world_tile_info_data_offset: i32,
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
                engine_version_recorded: 0,
                engine_version_compatible: 0,
                chunk_ids: Vec::new(),
                package_source: 0,
                folder_name: None,
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
                world_tile_info_data_offset: 0,
            }
        }

        pub fn parse_data(&mut self) -> Result<(), Error> {
            println!("Parsing data...");

            // reuseable buffers for reading
            let mut buf4 = [0u8; 4];
            //let mut buf8 = [0u8; 8];

            // seek to start
            self.cursor.seek(SeekFrom::Start(0))?;

            // read and check magic
            self.cursor.read_exact(&mut buf4)?;
            if u32::from_be_bytes(buf4) != UE4_ASSET_MAGIC {
                return Err(Error::new(
                    ErrorKind::Other,
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
            // TODO actually read this
            if (self.legacy_file_version <= -2) {
                // TODO: support for enum-based custom versions

                // read custom version count
                self.cursor.read_exact(&mut buf4)?;
                let mut num_custom_versions = i32::from_le_bytes(buf4);
                println!("Num custom versions: {}", num_custom_versions);

                for _ in 0..num_custom_versions {
                    // read custom version
                    // 16 byte guid + 4 byte version

                    // for now skip 20 bytes
                    self.cursor.seek(SeekFrom::Current(20))?;
                }
            }

            /*println!(
                "data (len: {:?}): {:02X?}",
                self.cursor.get_ref().len(),
                self.cursor.get_ref()
            );*/

            Ok(())
        }
    }
}
