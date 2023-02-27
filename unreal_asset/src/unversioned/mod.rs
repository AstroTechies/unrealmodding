use std::{hash::Hash, io::Cursor};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    containers::indexed_map::IndexedMap,
    cursor_ext::ReadExt,
    error::{Error, UsmapError},
};

use self::{properties::UsmapProperty, usmap_reader::UsmapReader};

pub mod properties;
pub mod usmap_reader;
pub mod usmap_writer;

#[cfg(feature = "oodle")]
pub(crate) mod oodle;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum UsmapVersion {
    INITIAL,
    LATEST,
    LATEST_PLUS_ONE,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECompressionMethod {
    None,
    Oodle,
    Brotli,

    Unknown = 0xFF,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapSchema {
    pub name: String,
    pub super_type: String,
    pub prop_count: u16,
    pub properties: Vec<UsmapProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usmap {
    pub version: UsmapVersion,
    pub name_map: Vec<String>,
    pub enum_map: IndexedMap<String, Vec<String>>,
    pub schemas: IndexedMap<String, UsmapSchema>,

    cursor: Cursor<Vec<u8>>,
}

impl Usmap {
    const ASSET_MAGIC: u16 = u16::from_be_bytes([0xc4, 0x30]);

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

    pub fn parse_data(&mut self) -> Result<(), Error> {
        self.parse_header()?;

        let names_len = self.read_i32()?;
        for _ in 0..names_len {
            let name = self.read_string()?;
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
    fn read_i8(&mut self) -> Result<i8, std::io::Error> {
        self.cursor.read_i8()
    }

    fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        self.cursor.read_u8()
    }

    fn read_i16(&mut self) -> Result<i16, std::io::Error> {
        self.cursor.read_i16::<LittleEndian>()
    }

    fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        self.cursor.read_u16::<LittleEndian>()
    }

    fn read_i32(&mut self) -> Result<i32, std::io::Error> {
        self.cursor.read_i32::<LittleEndian>()
    }

    fn read_u32(&mut self) -> Result<u32, std::io::Error> {
        self.cursor.read_u32::<LittleEndian>()
    }

    fn read_i64(&mut self) -> Result<i64, std::io::Error> {
        self.cursor.read_i64::<LittleEndian>()
    }

    fn read_u64(&mut self) -> Result<u64, std::io::Error> {
        self.cursor.read_u64::<LittleEndian>()
    }

    fn read_f32(&mut self) -> Result<f32, std::io::Error> {
        self.cursor.read_f32::<LittleEndian>()
    }

    fn read_f64(&mut self) -> Result<f64, std::io::Error> {
        self.cursor.read_f64::<LittleEndian>()
    }

    fn read_string(&mut self) -> Result<String, Error> {
        self.cursor.read_string().map(|e| e.unwrap_or_default())
    }

    fn read_name(&mut self) -> Result<String, std::io::Error> {
        let index = self.read_i32()?;
        if index < 0 {
            return Ok("".to_string());
        }
        Ok(self.name_map[index as usize].clone())
    }
}
