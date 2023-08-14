//! .utoc chunks

use std::io::{Cursor, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, BE, LE};

use crate::error::Error;

/// IoStore chunk type < UE5
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum EIoChunkType {
    /// Invalid
    Invalid,
    /// Install manifest
    InstallManifest,
    /// Export bundle data
    ExportBundleData,
    /// Bulk data
    BulkData,
    /// Optional bulk data
    OptionalBulkData,
    /// Memory mapped bulk data
    MemoryMappedBulkData,
    /// Loader global meta
    LoaderGlobalMeta,
    /// Loader initial load meta
    LoaderInitialLoadMeta,
    /// Loader global names
    LoaderGlobalNames,
    /// Loader global name hashes
    LoaderGlobalNameHashes,
    /// Container header
    ContainerHeader,
}

impl ToString for EIoChunkType {
    fn to_string(&self) -> String {
        match self {
            EIoChunkType::Invalid => String::from("Invalid"),
            EIoChunkType::InstallManifest => String::from("InstallManifest"),
            EIoChunkType::ExportBundleData => String::from("ExportBundleData"),
            EIoChunkType::BulkData => String::from("BulkData"),
            EIoChunkType::OptionalBulkData => String::from("OptionalBulkData"),
            EIoChunkType::MemoryMappedBulkData => String::from("MemoryMappedBulkData"),
            EIoChunkType::LoaderGlobalMeta => String::from("LoaderGlobalMeta"),
            EIoChunkType::LoaderInitialLoadMeta => String::from("LoaderInitialLoadMeta"),
            EIoChunkType::LoaderGlobalNames => String::from("LoaderGlobalNames"),
            EIoChunkType::LoaderGlobalNameHashes => String::from("LoaderGlobalNameHashes"),
            EIoChunkType::ContainerHeader => String::from("ContainerHeader"),
        }
    }
}

/// IoStore chunk type >= UE5
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum EIoChunkType5 {
    /// Invalid
    Invalid = 0,
    /// Export bundle data
    ExportBundleData = 1,
    /// Bulk data
    BulkData = 2,
    /// Optional bulk data
    OptionalBulkData = 3,
    /// Memory mapped bulk data
    MemoryMappedBulkData = 4,
    /// Script objects
    ScriptObjects = 5,
    /// Container header
    ContainerHeader = 6,
    /// External file
    ExternalFile = 7,
    /// Shader code library
    ShaderCodeLibrary = 8,
    /// Shader code
    ShaderCode = 9,
    /// Package store entry
    PackageStoreEntry = 10,
    /// Derived data
    DerivedData = 11,
    /// Editor derived data
    EditorDerivedData = 12,
}

impl ToString for EIoChunkType5 {
    fn to_string(&self) -> String {
        match self {
            EIoChunkType5::Invalid => String::from("Invalid"),
            EIoChunkType5::ExportBundleData => String::from("ExportBundleData"),
            EIoChunkType5::BulkData => String::from("BulkData"),
            EIoChunkType5::OptionalBulkData => String::from("OptionalBulkData"),
            EIoChunkType5::MemoryMappedBulkData => String::from("MemoryMappedBulkData"),
            EIoChunkType5::ScriptObjects => String::from("ScriptObjects"),
            EIoChunkType5::ContainerHeader => String::from("ContainerHeader"),
            EIoChunkType5::ExternalFile => String::from("ExternalFile"),
            EIoChunkType5::ShaderCodeLibrary => String::from("ShaderCodeLibrary"),
            EIoChunkType5::ShaderCode => String::from("ShaderCode"),
            EIoChunkType5::PackageStoreEntry => String::from("PackageStoreEntry"),
            EIoChunkType5::DerivedData => String::from("DerivedData"),
            EIoChunkType5::EditorDerivedData => String::from("EditorDerivedData"),
        }
    }
}

/// IoStore chunk id
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoChunkId {
    /// Chunk id
    pub chunk_id: u64,
    /// Chunk index
    pub chunk_index: u16,
    /// Chunk type
    pub chunk_type: u8,
}

impl IoChunkId {
    /// Create a new chunk id
    pub fn new(chunk_id: u64, chunk_index: u16, chunk_type: u8) -> IoChunkId {
        IoChunkId {
            chunk_id,
            chunk_index,
            chunk_type,
        }
    }

    /// Read `IoChunkId` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let chunk_id = reader.read_u64::<LE>()?;
        let chunk_index = reader.read_u16::<BE>()?;

        let _ = reader.read_u8()?; // padding

        let chunk_type = reader.read_u8()?;

        Ok(IoChunkId {
            chunk_id,
            chunk_index,
            chunk_type,
        })
    }

    /// Write `IoChunkId` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u64::<LE>(self.chunk_id)?;
        writer.write_u16::<BE>(self.chunk_index)?;

        writer.write_u8(0)?; // padding

        writer.write_u8(self.chunk_type)?;

        Ok(())
    }

    /// Hash `IoChunkId` for searching in the .utoc file
    pub fn hash(&self, seed: i32) -> Result<u64, Error> {
        let mut cursor = Cursor::new(Vec::new());
        self.write(&mut cursor)?;

        let data = cursor.into_inner();
        let mut hash = match seed {
            0 => 0xcbf29ce484222325,
            _ => seed as u64,
        };

        for byte in data {
            hash = hash.overflowing_mul(0x00000100000001B3).0 ^ byte as u64;
        }

        Ok(hash)
    }
}

/// IoStore chunk hash
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoChunkHash {
    /// Value
    pub value: [u8; 32],
}

impl IoChunkHash {
    /// Read `IoChunkHash` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut value = [0u8; 32];
        reader.read_exact(&mut value)?;
        Ok(IoChunkHash { value })
    }

    /// Write `IoChunkHash` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.value)?;
        Ok(())
    }
}
