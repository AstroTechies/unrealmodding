//! Allows reading unversioned assets using mappings

use std::hash::Hash;
use std::io::{self, Cursor, Seek, SeekFrom};

use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use unreal_helpers::UnrealReadExt;

use crate::containers::indexed_map::IndexedMap;
use crate::crc;
use crate::error::{Error, UsmapError};
use crate::types::FName;

use self::ancestry::Ancestry;
use self::usmap_trait::UsmapTrait;
use self::{properties::UsmapProperty, usmap_reader::UsmapReader};

pub mod ancestry;
pub mod header;
pub mod properties;
pub mod usmap_reader;
pub mod usmap_trait;
pub mod usmap_writer;

#[cfg(feature = "oodle")]
pub(crate) mod oodle;

/// Usmap file version
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum UsmapVersion {
    /// Initial
    Initial,
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECompressionMethod {
    /// None
    None,
    /// Oodle
    Oodle,
    /// Brotli
    Brotli,

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
    pub super_type: Option<String>,
    /// Properties count
    pub prop_count: u16,
    /// Module path
    pub module_path: Option<String>,
    /// Properties
    pub properties: IndexedMap<UsmapPropertyKey, UsmapProperty>,
}

impl UsmapSchema {
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
    pub version: UsmapVersion,
    /// Name map
    pub name_map: Vec<String>,
    /// Enum map
    pub enum_map: IndexedMap<String, Vec<String>>,
    /// Schemas
    pub schemas: IndexedMap<String, UsmapSchema>,
    /// Extension version
    pub extension_version: UsmapExtensionVersion,
    /// Pre-computed cityhash64 map for relevant strings
    pub cityhash64_map: IndexedMap<u64, String>,

    /// Binary cursor
    cursor: Cursor<Vec<u8>>,
}

impl Usmap {
    const ASSET_MAGIC: u16 = u16::from_be_bytes([0xc4, 0x30]);

    /// Parse usmap file header
    fn parse_header(&mut self) -> Result<(), Error> {
        let magic = self.cursor.read_u16::<LE>()?;
        if magic != Usmap::ASSET_MAGIC {
            return Err(Error::invalid_file(
                "File is not a valid usmap file".to_string(),
            ));
        }

        let version: UsmapVersion = UsmapVersion::try_from(self.cursor.read_u8()?)?;
        self.version = version;

        let compression = self.cursor.read_u8()?;
        let compression_method: ECompressionMethod = ECompressionMethod::try_from(compression)?;

        let compressed_size = self.cursor.read_u32::<LE>()?;
        let decompressed_size = self.cursor.read_u32::<LE>()?;

        match compression_method {
            ECompressionMethod::None => {
                if compressed_size != decompressed_size {
                    return Err(Error::invalid_file(
                        "compressed_size != decompressed size on an uncompressed file".to_string(),
                    ));
                }
            }
            ECompressionMethod::Oodle => {
                #[cfg(not(feature = "oodle"))]
                return Err(UsmapError::unsupported_compression(compression).into());

                #[cfg(feature = "oodle")]
                {
                    let compressed = vec![0u8; compressed_size as usize];

                    let decompressed = oodle::decompress(
                        &compressed,
                        compressed_size as u64,
                        decompressed_size as u64,
                    )
                    .ok_or_else(|| UsmapError::invalid_compression_data())?;

                    self.cursor = Cursor::new(decompressed);
                }
            }
            _ => return Err(UsmapError::unsupported_compression(compression).into()),
        }

        Ok(())
    }

    /// Add a cityhash64 map entry to the precomputed map
    fn add_cityhash64_map_entry(&mut self, entry: &str) -> Result<(), Error> {
        let hash = crc::generate_import_hash_from_object_path(entry);
        if let Some(existing_entry) = self.cityhash64_map.get_by_key(&hash) {
            if crc::to_lower_string(existing_entry) == crc::to_lower_string(entry) {
                return Ok(());
            }
            return Err(UsmapError::cityhash64_collision(hash, entry.to_string()).into());
        }
        self.cityhash64_map.insert(hash, entry.to_string());
        Ok(())
    }

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
            let Some(ref super_type) = schema.super_type else {
                break;
            };
            schema_name = super_type.as_str();
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
        let mut optional_schema_name = ancestry.get_parent().map(|e| e.content.clone());

        let mut global_index = 0;
        loop {
            let Some(schema_name) = optional_schema_name else {
                break;
            };

            let Some(schema) = self.schemas.get_by_key(&schema_name) else {
                break;
            };

            if let Some(property) = schema.get_property(&property_name.content, duplication_index) {
                global_index += property.schema_index as u32;
                return Some((property, global_index));
            }

            global_index += schema.prop_count as u32;

            optional_schema_name = schema.super_type.clone();
        }

        // this name is not an actual property name, but an array index
        let Ok(_) = property_name.content.parse::<u32>() else {
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
    pub fn parse_data(&mut self) -> Result<(), Error> {
        self.parse_header()?;

        let names_len = self.read_i32()?;
        for _ in 0..names_len {
            let name = self.read_fstring()?.unwrap_or_default();
            self.name_map.push(name);
        }

        let enums_len = self.read_i32()?;
        for _ in 0..enums_len {
            let enum_name = self.read_name()?.ok_or_else(UsmapError::name_none)?;

            let enum_entries_len = self.read_u8()?;
            for _ in 0..enum_entries_len {
                let name = self.read_name()?.ok_or_else(UsmapError::name_none)?;
                self.enum_map
                    .entry(enum_name.clone())
                    .or_insert_with(Vec::new)
                    .push(name);
            }
        }

        let schemas_len = self.read_i32()?;
        for _ in 0..schemas_len {
            let schema_name = self.read_name()?.ok_or_else(UsmapError::name_none)?;
            let schema_super_type = self.read_name()?;
            let num_props = self.read_u16()?;
            let serializable_prop_count = self.read_u16()?;

            let mut properties = IndexedMap::new();

            for _ in 0..serializable_prop_count {
                let original_property = UsmapProperty::new(self)?;

                for k in 0..original_property.array_size as u16 {
                    let mut property = original_property.clone();
                    property.schema_index = original_property.schema_index + k;
                    property.array_index = k;

                    properties.insert(
                        (
                            property.name.clone().unwrap_or_default(),
                            property.array_index as u32,
                        ),
                        property,
                    );
                }
            }

            self.schemas.insert(
                schema_name.clone(),
                UsmapSchema {
                    name: schema_name,
                    super_type: schema_super_type,
                    prop_count: num_props,
                    module_path: None,
                    properties,
                },
            );
        }

        if self.stream_length()? > self.position() {
            self.extension_version = UsmapExtensionVersion::from_bits(self.read_u32()?)
                .ok_or_else(|| Error::invalid_file("Invalid object flags".to_string()))?;

            if self
                .extension_version
                .contains(UsmapExtensionVersion::PATHS)
            {
                let num_module_paths = self.read_u16()?;
                let mut module_paths = Vec::with_capacity(num_module_paths as usize);

                for _ in 0..num_module_paths {
                    module_paths.push(self.read_fstring()?);
                }

                for i in 0..self.schemas.len() {
                    let index = match num_module_paths > u8::MAX as u16 {
                        true => self.read_u16()? as usize,
                        false => self.read_u8()? as usize,
                    };

                    let schema = self.schemas.get_by_index_mut(i).unwrap();
                    schema.module_path = module_paths[index].clone();
                    let entry_name =
                        module_paths[index].clone().unwrap_or_default() + "." + &schema.name;
                    self.add_cityhash64_map_entry(&entry_name)?;
                }
            }
        }

        Ok(())
    }

    /// Create a new usmap file
    pub fn new(cursor: Cursor<Vec<u8>>) -> Result<Self, Error> {
        Ok(Usmap {
            version: UsmapVersion::Initial,
            name_map: Vec::new(),
            enum_map: IndexedMap::new(),
            schemas: IndexedMap::new(),
            extension_version: UsmapExtensionVersion::NONE,
            cityhash64_map: IndexedMap::new(),
            cursor,
        })
    }
}

impl UsmapTrait for Usmap {
    fn position(&mut self) -> u64 {
        self.cursor.position()
    }

    fn stream_length(&mut self) -> io::Result<u64> {
        let original_pos = self.cursor.position();

        self.cursor.seek(SeekFrom::End(0))?;
        let length = self.cursor.position();

        self.cursor.seek(SeekFrom::Start(original_pos))?;
        Ok(length)
    }
}

impl UsmapReader for Usmap {
    fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor.read_i8()
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<LE>()
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<LE>()
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        self.cursor.read_i32::<LE>()
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<LE>()
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor.read_i64::<LE>()
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor.read_u64::<LE>()
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        self.cursor.read_f32::<LE>()
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        self.cursor.read_f64::<LE>()
    }

    fn read_fstring(&mut self) -> Result<Option<String>, Error> {
        Ok(self.cursor.read_fstring()?)
    }

    fn read_name(&mut self) -> io::Result<Option<String>> {
        let index = self.read_i32()?;
        if index < 0 {
            return Ok(None);
        }
        Ok(Some(self.name_map[index as usize].clone()))
    }
}
