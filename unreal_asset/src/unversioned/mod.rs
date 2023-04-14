//! Allows reading unversioned assets using mappings

use std::hash::Hash;
use std::io::{self, Cursor};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use unreal_helpers::UnrealReadExt;

use crate::containers::indexed_map::IndexedMap;
use crate::error::{Error, UsmapError};

use self::{properties::UsmapProperty, usmap_reader::UsmapReader};

pub mod properties;
pub mod usmap_reader;
pub mod usmap_writer;

#[cfg(feature = "oodle")]
pub(crate) mod oodle;

/// Usmap file version
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum UsmapVersion {
    /// Initial
    INITIAL,
    /// Latest
    LATEST,
    /// Latest plus one
    LATEST_PLUS_ONE,
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

/// Usmap file schema
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapSchema {
    /// Name
    pub name: String,
    /// Super type
    pub super_type: String,
    /// Properties count
    pub prop_count: u16,
    /// Properties
    pub properties: Vec<UsmapProperty>,
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

    /// Binary cursor
    cursor: Cursor<Vec<u8>>,
}

impl Usmap {
    const ASSET_MAGIC: u16 = u16::from_be_bytes([0xc4, 0x30]);

    /// Parse usmap file header
    fn parse_header(&mut self) -> Result<(), Error> {
        let magic = self.cursor.read_u16::<LittleEndian>()?;
        if magic != Usmap::ASSET_MAGIC {
            return Err(Error::invalid_file(
                "File is not a valid usmap file".to_string(),
            ));
        }

        let version: UsmapVersion = UsmapVersion::try_from(self.cursor.read_u8()?)?;
        self.version = version;

        let compression = self.cursor.read_u8()?;
        let compression_method: ECompressionMethod = ECompressionMethod::try_from(compression)?;

        let compressed_size = self.cursor.read_u32::<LittleEndian>()?;
        let decompressed_size = self.cursor.read_u32::<LittleEndian>()?;

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
            let enum_name = self.read_name()?;

            let enum_entries_len = self.read_u8()?;
            for _ in 0..enum_entries_len {
                let name = self.read_name()?;
                self.enum_map
                    .entry(enum_name.clone())
                    .or_insert_with(Vec::new)
                    .push(name);
            }
        }

        let schemas_len = self.read_i32()?;
        for _ in 0..schemas_len {
            let schema_name = self.read_name()?;
            let schema_super_type = self.read_name()?;
            let num_props = self.read_u16()?;
            let serializable_prop_count = self.read_u16()?;

            let mut properties = Vec::new();

            for _ in 0..serializable_prop_count {
                let property = UsmapProperty::new(self)?;
                properties.push(property);
            }

            self.schemas.insert(
                schema_name.clone(),
                UsmapSchema {
                    name: schema_name,
                    super_type: schema_super_type,
                    prop_count: num_props,
                    properties,
                },
            );
        }

        Ok(())
    }

    /// Create a new usmap file
    pub fn new(cursor: Cursor<Vec<u8>>) -> Result<Self, Error> {
        Ok(Usmap {
            version: UsmapVersion::INITIAL,
            name_map: Vec::new(),
            enum_map: IndexedMap::new(),
            schemas: IndexedMap::new(),
            cursor,
        })
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
        self.cursor.read_i16::<LittleEndian>()
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<LittleEndian>()
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        self.cursor.read_i32::<LittleEndian>()
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<LittleEndian>()
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor.read_i64::<LittleEndian>()
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor.read_u64::<LittleEndian>()
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        self.cursor.read_f32::<LittleEndian>()
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        self.cursor.read_f64::<LittleEndian>()
    }

    fn read_fstring(&mut self) -> Result<Option<String>, Error> {
        Ok(self.cursor.read_fstring()?)
    }

    fn read_name(&mut self) -> io::Result<String> {
        let index = self.read_i32()?;
        if index < 0 {
            return Ok("".to_string());
        }
        Ok(self.name_map[index as usize].clone())
    }
}
