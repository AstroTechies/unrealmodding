//! Allows reading unversioned assets using mappings

use std::hash::Hash;
use std::io::{Cursor, Read, Seek};

use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::asset::name_map::NameMap;
use crate::containers::chain::Chain;
use crate::containers::indexed_map::IndexedMap;
use crate::custom_version::CustomVersion;
use crate::error::{Error, UsmapError};
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::archive_reader::ArchiveReader;
use crate::reader::archive_trait::ArchiveTrait;
use crate::reader::raw_reader::RawReader;
use crate::types::fname::FName;

use self::ancestry::Ancestry;
use self::properties::UsmapProperty;
use self::usmap_reader::UsmapReader;

pub mod ancestry;
pub mod header;
#[cfg(feature = "oodle")]
pub(crate) mod oodle;
pub mod properties;
pub mod usmap_reader;
pub mod usmap_writer;

/// Usmap file version
#[derive(
    Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum EUsmapVersion {
    /// Initial
    Initial,

    /// Adds package versioning to aid with compatibililty
    PackageVersioning,

    /// Latest
    Latest,
    /// Latest plus one
    LatestPlusOne,
}

bitflags! {
    /// Usmap extension version
    pub struct UsmapExtensionVersion : u32 {
        /// No extension data is present
        const NONE = 0;
        /// Module path information is present
        const PATHS = 1;
    }
}

/// Usmap file compression method
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EUsmapCompressionMethod {
    /// None
    None,
    /// Oodle
    Oodle,
    /// Brotli
    Brotli,
    /// ZStandard
    ZStandard,

    /// Unknown
    Unknown = 0xFF,
}

type UsmapPropertyKey = (String, u32);

/// Usmap file schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsmapSchema {
    /// Name
    pub name: String,
    /// Super type
    pub super_type: String,
    /// Properties count
    pub prop_count: u16,
    /// Module path
    pub module_path: Option<String>,
    /// Properties
    pub properties: IndexedMap<UsmapPropertyKey, UsmapProperty>,
}

impl UsmapSchema {
    /// Read a `UsmapSchema` from an archive
    pub fn read<'parent_reader, 'asset, R: ArchiveReader>(
        reader: &mut UsmapReader<'parent_reader, 'asset, R>,
    ) -> Result<UsmapSchema, Error> {
        let name = reader.read_name()?;
        let super_type = reader.read_name()?;

        let prop_count = reader.read_u16::<LE>()?;
        let serializable_property_count = reader.read_u16::<LE>()?;

        let mut properties = IndexedMap::with_capacity(prop_count as usize);

        for _ in 0..serializable_property_count {
            let property = UsmapProperty::new(reader)?;

            for j in 0..property.array_size {
                let mut property = property.clone();
                property.array_index = j as u16;
                property.schema_index += j as u16;

                properties.insert(
                    (property.name.clone(), property.schema_index as u32),
                    property,
                );
            }
        }

        Ok(UsmapSchema {
            name,
            super_type,
            prop_count,
            module_path: None,
            properties,
        })
    }

    /// Gets a usmap property
    pub fn get_property(&self, name: &str, duplication_index: u32) -> Option<&UsmapProperty> {
        // todo: remove to_string
        self.properties
            .get_by_key(&(name.to_string(), duplication_index))
    }
}

/// Usmap file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usmap {
    /// File version
    pub version: EUsmapVersion,
    /// Name map
    pub name_map: Vec<String>,
    /// Enum map
    pub enum_map: IndexedMap<String, Vec<String>>,
    /// Schemas
    pub schemas: IndexedMap<String, UsmapSchema>,
    /// Extension version
    pub extension_version: UsmapExtensionVersion,
    /// UE4 object version
    pub object_version: ObjectVersion,
    /// UE5 object version
    pub object_version_ue5: ObjectVersionUE5,
    /// Custom version container
    pub custom_versions: Vec<CustomVersion>,
    /// Compression method
    pub compression_method: EUsmapCompressionMethod,
    /// Net CL
    pub net_cl: u32,
}

impl Usmap {
    const ASSET_MAGIC: u16 = u16::from_be_bytes([0xc4, 0x30]);

    /// Gets usmap property for a given property name + ancestry
    pub fn get_property(
        &self,
        property_name: &FName,
        ancestry: &Ancestry,
    ) -> Option<&UsmapProperty> {
        self.get_property_with_duplication_index(property_name, ancestry, 0)
            .map(|(property, _)| property)
    }

    /// Gets all usmap mappings for a given schema
    pub fn get_all_properties<'name>(
        &'name self,
        mut schema_name: &'name str,
    ) -> Vec<&UsmapProperty> {
        let mut properties = Vec::new();

        while let Some(schema) = self.schemas.get_by_key(schema_name) {
            properties.extend(schema.properties.values());
            schema_name = schema.super_type.as_str();
        }

        properties
    }

    /// Gets usmap property and it's "global" index for a given proeprty name + ancestry with a duplication index
    pub fn get_property_with_duplication_index(
        &self,
        property_name: &FName,
        ancestry: &Ancestry,
        duplication_index: u32,
    ) -> Option<(&UsmapProperty, u32)> {
        let mut optional_schema_name = ancestry.get_parent().map(|e| e.get_content());

        let mut global_index = 0;
        loop {
            let Some(schema_name) = optional_schema_name else {
                break;
            };

            let Some(schema) = self.schemas.get_by_key(&schema_name) else {
                break;
            };

            if let Some(property) =
                schema.get_property(&property_name.get_content(), duplication_index)
            {
                global_index += property.schema_index as u32;
                return Some((property, global_index));
            }

            global_index += schema.prop_count as u32;

            optional_schema_name = Some(schema.super_type.clone());
        }

        // this name is not an actual property name, but an array index
        let Ok(_) = property_name.get_content().parse::<u32>() else {
            return None;
        };

        let Some(parent) = ancestry.get_parent() else {
            return None;
        };

        self.get_property_with_duplication_index(
            parent,
            &ancestry.without_parent(),
            duplication_index,
        )
    }

    /// Parse usmap file
    pub fn parse_data<C: Read + Seek>(&mut self, cursor: C) -> Result<(), Error> {
        let mut reader = RawReader::new(
            Chain::new(cursor, None),
            ObjectVersion::UNKNOWN,
            ObjectVersionUE5::UNKNOWN,
            false,
            NameMap::new(),
        );

        let magic = reader.read_u16::<LE>()?;
        if magic != Self::ASSET_MAGIC {
            return Err(Error::invalid_file(
                "File is not a valid usmap file".to_string(),
            ));
        }

        let usmap_version = EUsmapVersion::try_from(reader.read_u8()?)?;

        let mut has_versioning = usmap_version >= EUsmapVersion::PackageVersioning;
        if has_versioning {
            has_versioning = reader.read_bool()?;
        }

        if has_versioning {
            self.object_version = ObjectVersion::try_from(reader.read_i32::<LE>()?)?;
            self.object_version_ue5 = ObjectVersionUE5::try_from(reader.read_i32::<LE>()?)?;
            self.custom_versions = reader.read_array(|e| CustomVersion::read(e))?;
            self.net_cl = reader.read_u32::<LE>()?;
        }

        self.compression_method = EUsmapCompressionMethod::try_from(reader.read_u8()?)?;

        let compressed_size = reader.read_u32::<LE>()?;
        let decompressed_size = reader.read_u32::<LE>()?;

        let mut compressed_data = vec![0u8; compressed_size as usize];
        reader.read_exact(&mut compressed_data);

        let data = match self.compression_method {
            EUsmapCompressionMethod::None => {
                if compressed_size != decompressed_size {
                    return Err(Error::invalid_file(
                        "compressed_size != decompressed size on an uncompressed file".to_string(),
                    ));
                }

                compressed_data
            }
            EUsmapCompressionMethod::Brotli => {
                let mut decompressed_data = Cursor::new(vec![0u8; decompressed_size as usize]);
                brotli::BrotliDecompress(&mut Cursor::new(compressed_data), &mut decompressed_data);
                decompressed_data.into_inner()
            }
            EUsmapCompressionMethod::ZStandard => {
                let mut decompressed_data = Cursor::new(vec![0u8; decompressed_size as usize]);
                zstd::stream::copy_decode(
                    &mut Cursor::new(compressed_data),
                    &mut decompressed_data,
                )?;
                decompressed_data.into_inner()
            }
            EUsmapCompressionMethod::Oodle => {
                #[cfg(not(feature = "oodle"))]
                return Err(
                    UsmapError::unsupported_compression(self.compression_method as u8).into(),
                );

                #[cfg(feature = "oodle")]
                {
                    let compressed = vec![0u8; compressed_size as usize];

                    let decompressed = oodle::decompress(
                        &compressed,
                        compressed_size as u64,
                        decompressed_size as u64,
                    )
                    .ok_or_else(|| UsmapError::invalid_compression_data())?;

                    decompressed
                }
            }
            EUsmapCompressionMethod::Unknown => {
                return Err(
                    UsmapError::unsupported_compression(self.compression_method as u8).into(),
                );
            }
        };

        let mut reader = RawReader::new(
            Chain::new(Cursor::new(data), None),
            self.object_version,
            self.object_version_ue5,
            false,
            NameMap::new(),
        );

        self.name_map = reader.read_array(|reader| {
            let name_length = reader.read_u8()?;
            let mut buf = vec![0u8; name_length as usize - 1];
            reader.read_exact(&mut buf)?;
            Ok(String::from_utf8(buf)?)
        })?;

        let enum_len = reader.read_u32::<LE>()?;
        self.enum_map = IndexedMap::with_capacity(enum_len as usize);

        let mut reader = UsmapReader::new(&mut reader, &self.name_map, &self.custom_versions);

        for _ in 0..enum_len {
            let enum_name = reader.read_name()?;

            let enum_names_len = reader.read_u8()?;
            let mut enum_names = Vec::with_capacity(enum_names_len as usize);

            for _ in 0..enum_names_len {
                enum_names.push(reader.read_name()?);
            }

            self.enum_map.insert(enum_name, enum_names);
        }

        let schemas_len = reader.read_u32::<LE>()?;
        self.schemas = IndexedMap::with_capacity(schemas_len as usize);

        for _ in 0..schemas_len {
            let schema = UsmapSchema::read(&mut reader)?;
            self.schemas.insert(schema.name.clone(), schema);
        }

        // read extensions

        if reader.data_length()? > reader.position() {
            self.extension_version = UsmapExtensionVersion::from_bits(reader.read_u32::<LE>()?)
                .ok_or_else(|| Error::invalid_file("Invalid extension version".to_string()))?;

            if self
                .extension_version
                .contains(UsmapExtensionVersion::PATHS)
            {
                let num_module_paths = reader.read_u16::<LE>()?;
                let module_paths = reader
                    .read_array_with_length(num_module_paths as i32, |reader| {
                        Ok(reader.read_fstring()?.unwrap_or_default())
                    })?;

                for (_, _, schema) in self.schemas.iter_mut() {
                    let index = match num_module_paths > u8::MAX as u16 {
                        true => reader.read_u16::<LE>()?,
                        false => reader.read_u8()? as u16,
                    };
                    schema.module_path = Some(module_paths[index as usize].clone());
                }
            }
        }

        Ok(())
    }

    /// Create a new usmap file
    pub fn new(cursor: Cursor<Vec<u8>>) -> Result<Self, Error> {
        let mut usmap = Usmap {
            version: EUsmapVersion::Initial,
            name_map: Vec::new(),
            enum_map: IndexedMap::new(),
            schemas: IndexedMap::new(),
            extension_version: UsmapExtensionVersion::NONE,
            object_version: ObjectVersion::UNKNOWN,
            object_version_ue5: ObjectVersionUE5::UNKNOWN,
            custom_versions: Vec::new(),
            compression_method: EUsmapCompressionMethod::None,
            net_cl: 0,
        };
        usmap.parse_data(cursor)?;
        Ok(usmap)
    }
}
