#![deny(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(mismatched_lifetime_syntaxes)]
#![allow(clippy::needless_lifetimes)]

//! Unreal Asset Registry
//!
//! Asset Registry is used for storing information about assets
//! The information from Asset Registry is primarily used in Content Browser,
//! but some games might require modifying it before your assets will get loaded

use std::io::{Cursor, Seek, SeekFrom};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    containers::{NameMap, SharedResource},
    crc,
    custom_version::FAssetRegistryVersionType,
    error::RegistryError,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::{ArchiveReader, ArchiveTrait, ArchiveWriter, RawWriter},
    types::{PackageIndex, PackageIndexTrait},
    Error,
};

pub(crate) mod name_table_reader;
pub(crate) mod name_table_writer;
pub mod objects;

use name_table_reader::NameTableReader;
use name_table_writer::NameTableWriter;
use objects::{
    asset_data::AssetData, asset_package_data::AssetPackageData, depends_node::DependsNode,
};

// reexports for tests
#[doc(hidden)]
pub mod unreal_asset {
    pub use crate as registry;
    pub use unreal_asset_base::*;
}

/// Asset registry state
#[derive(Debug)]
pub struct AssetRegistryState {
    /// Assets data
    pub assets_data: Vec<AssetData>,
    /// Depends nodes
    pub depends_nodes: Vec<DependsNode>,
    /// Package data
    pub package_data: Vec<AssetPackageData>,

    /// Name map
    name_map: Option<SharedResource<NameMap>>,
    /// Object version
    object_version: ObjectVersion,
    /// UE5 Object version
    object_version_ue5: ObjectVersionUE5,
    /// Asset registry version
    version: FAssetRegistryVersionType,
}

impl AssetRegistryState {
    /// Read an `AssetRegistryState` from an asset
    fn load<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        version: FAssetRegistryVersionType,
        assets_data: &mut Vec<AssetData>,
        depends_nodes: &mut Vec<DependsNode>,
        package_data: &mut Vec<AssetPackageData>,
    ) -> Result<(), Error> {
        *assets_data = asset.read_array(|asset: &mut Reader| AssetData::new(asset, version))?;

        if version < FAssetRegistryVersionType::AddedDependencyFlags {
            let local_num_depends_nodes = asset.read_i32::<LE>()?;
            *depends_nodes = Vec::with_capacity(local_num_depends_nodes as usize);

            for i in 0..local_num_depends_nodes {
                depends_nodes.push(DependsNode::new(i, version));
            }
            let depends_nodes_copy = depends_nodes.clone();

            if local_num_depends_nodes > 0 {
                for depends_node in depends_nodes {
                    depends_node.load_dependencies_before_flags(asset, &depends_nodes_copy)?;
                }
            }
        } else {
            let dependency_section_size = asset.read_i64::<LE>()?;
            let dependency_section_end = asset.position() + dependency_section_size as u64;
            let local_num_depends_nodes = asset.read_i32::<LE>()?;

            *depends_nodes = Vec::with_capacity(local_num_depends_nodes as usize);
            for i in 0..local_num_depends_nodes {
                depends_nodes.push(DependsNode::new(i, version));
            }

            let assets_data_copy = depends_nodes.clone();
            if local_num_depends_nodes > 0 {
                for depends_node in depends_nodes {
                    depends_node.load_dependencies(asset, &assets_data_copy)?;
                }
            }

            asset.set_position(dependency_section_end)?;
        }

        *package_data =
            asset.read_array(|asset: &mut Reader| AssetPackageData::new(asset, version))?;

        Ok(())
    }

    /// Write an `AssetRegistryState` to an asset
    fn write_data<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        writer.write_i32::<LE>(self.assets_data.len() as i32)?;
        for asset_data in &self.assets_data {
            asset_data.write(writer)?;
        }

        if self.version < FAssetRegistryVersionType::AddedDependencyFlags {
            writer.write_i32::<LE>(self.depends_nodes.len() as i32)?;

            for depends_node in &self.depends_nodes {
                depends_node.save_dependencies_before_flags(writer)?;
            }
        } else {
            let pos = writer.position();
            writer.write_i64::<LE>(0)?;
            writer.write_i32::<LE>(self.depends_nodes.len() as i32)?;

            for depends_node in &self.depends_nodes {
                depends_node.save_dependencies(writer)?;
            }

            let end_pos = writer.position();
            writer.set_position(pos)?;
            writer.write_i64::<LE>(end_pos as i64 - pos as i64)?;
            writer.set_position(end_pos)?;
        }

        writer.write_i32::<LE>(self.package_data.len() as i32)?;
        for package_data in &self.package_data {
            package_data.write(writer)?;
        }

        Ok(())
    }

    /// Reads asset registry from a Reader
    ///
    /// # Errors
    ///
    /// If an asset registry is invalid throws ['RegistryError']
    ///
    /// If there was an IO error during read throws ['Io']
    ///
    /// ['RegistryError']: error/enum.RegistryError.html
    /// ['Io']: error/enum.ErrorCode.html#variant.Ios
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{
    ///     fs::File,
    ///     io::{Cursor, Read},
    ///     path::Path,
    /// };
    ///
    /// # use unreal_asset_registry::unreal_asset;
    /// use unreal_asset::{
    ///     engine_version::{self, EngineVersion},
    ///     registry::AssetRegistryState,
    ///     reader::RawReader,
    ///     containers::{NameMap, Chain}
    /// };
    ///
    /// let mut file = File::open("AssetRegistry.bin").unwrap();
    /// let mut data = Vec::new();
    /// file.read_to_end(&mut data).unwrap();
    ///
    /// let cursor = Cursor::new(data);
    /// let (object_version, object_version_ue5) = engine_version::get_object_versions(EngineVersion::VER_UE4_25);
    /// let mut raw_reader = RawReader::new(Chain::new(cursor, None), object_version, object_version_ue5, false, NameMap::new());
    /// let asset_registry = AssetRegistryState::new(&mut raw_reader).unwrap();
    ///
    /// println!("{:#?}", asset_registry);
    /// ```
    pub fn new<Reader: ArchiveReader<PackageIndex>>(asset: &mut Reader) -> Result<Self, Error> {
        let version = FAssetRegistryVersionType::new(asset)?;
        let mut assets_data = Vec::new();
        let mut depends_nodes = Vec::new();
        let mut package_data = Vec::new();

        let mut name_map = None;

        if version < FAssetRegistryVersionType::RemovedMD5Hash {
            return Err(Error::invalid_file(format!(
                "Cannot read registry state before {}",
                version as i32
            )));
        } else if version < FAssetRegistryVersionType::FixedTags {
            // name table reader
            let mut name_table_reader = NameTableReader::new(asset)?;
            Self::load(
                &mut name_table_reader,
                version,
                &mut assets_data,
                &mut depends_nodes,
                &mut package_data,
            )?;
            name_map = Some(name_table_reader.name_map);
        } else {
            Self::load(
                asset,
                version,
                &mut assets_data,
                &mut depends_nodes,
                &mut package_data,
            )?;
        }

        Ok(Self {
            version,
            assets_data,
            depends_nodes,
            package_data,

            name_map,

            object_version: asset.get_object_version(),
            object_version_ue5: asset.get_object_version_ue5(),
        })
    }

    /// Writes asset registry to a binary cursor
    ///
    /// # Errors
    ///
    /// If this asset registry was modified in a way that makes it invalid throws ['RegistryError']
    ///
    /// If there is an IO error during write throws ['Io`] error.
    ///
    /// ['RegistryError']: error/enum.RegistryError.html
    /// ['Io']: error/enum.ErrorCode.html#variant.Io
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{
    ///     fs::File,
    ///     io::{Cursor, Read},
    ///     path::Path,
    /// };
    ///
    /// # use unreal_asset_registry::unreal_asset;
    /// use unreal_asset::{
    ///     engine_version::{self, EngineVersion},
    ///     registry::AssetRegistryState,
    ///     reader::RawReader,
    ///     containers::{NameMap, Chain}
    /// };
    ///
    /// let mut file = File::open("AssetRegistry.bin").unwrap();
    /// let mut data = Vec::new();
    /// file.read_to_end(&mut data).unwrap();
    ///
    /// let cursor = Cursor::new(data);
    /// let (object_version, object_version_ue5) = engine_version::get_object_versions(EngineVersion::VER_UE4_25);
    /// let mut raw_reader = RawReader::new(Chain::new(cursor, None), object_version, object_version_ue5, false, NameMap::new());
    /// let asset_registry = AssetRegistryState::new(&mut raw_reader).unwrap();
    ///
    /// let mut cursor = Cursor::new(Vec::new());
    /// asset_registry.write(&mut cursor).unwrap();
    ///
    /// println!("{:#?}", cursor.get_ref());
    /// ```
    pub fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        let mut writer = RawWriter::new(
            cursor,
            self.object_version,
            self.object_version_ue5,
            false,
            NameMap::new(),
        );
        self.version.write(&mut writer)?;

        if self.version < FAssetRegistryVersionType::RemovedMD5Hash {
            return Err(Error::invalid_file(format!(
                "Cannot write registry state before {}",
                self.version as i32
            )));
        } else if self.version < FAssetRegistryVersionType::FixedTags {
            let pos = writer.position();
            writer.write_i64::<LE>(0)?;

            let name_map = self
                .name_map
                .as_ref()
                .ok_or_else(|| RegistryError::version("Name map".to_string(), self.version))?;

            let mut name_table_writer = NameTableWriter::new(&mut writer, name_map.clone());

            self.write_data(&mut name_table_writer)?;

            let offset = writer.position();
            writer.write_i32::<LE>(name_map.get_ref().get_name_map_index_list().len() as i32)?;
            for name in name_map.get_ref().get_name_map_index_list() {
                writer.write_fstring(Some(name))?;

                #[allow(clippy::single_match)]
                match writer.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
                    true => {
                        let hash = crc::generate_hash(name);
                        writer.write_u32::<LE>(hash)?;
                    }
                    false => {}
                }
            }

            let end = writer.position();

            writer.seek(SeekFrom::Start(pos))?;
            writer.write_i64::<LE>(offset as i64)?;
            writer.seek(SeekFrom::Start(end))?;
        } else {
            self.write_data(&mut writer)?;
        }

        Ok(())
    }

    /// Adds a name reference to the string lookup table
    pub fn add_name_reference(&mut self, string: &str, add_duplicates: bool) -> i32 {
        if let Some(ref mut name_map) = self.name_map {
            return name_map
                .get_mut()
                .add_name_reference(string.to_string(), add_duplicates);
        };

        0
    }

    /// Does the same as ['add_name_reference'] without adding duplicates
    pub fn add_fname(&mut self, string: &str) -> i32 {
        self.add_name_reference(string, false)
    }

    /// Gets current AssetRegistry version
    pub fn get_version(&self) -> FAssetRegistryVersionType {
        self.version
    }
}
