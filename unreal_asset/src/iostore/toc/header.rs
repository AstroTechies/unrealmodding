//! .utoc header

use std::{
    io::{Read, Seek, Write},
    mem::size_of,
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use unreal_helpers::{Guid, UnrealReadExt, UnrealWriteExt};

use crate::{
    error::{Error, IoStoreError},
    iostore::flags::EIoContainerFlags,
};

use super::{EIoStoreTocVersion, IoContainerId};

/// IoStore .utoc header
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreTocHeader {
    /// Version
    pub version: EIoStoreTocVersion,
    /// Reserved
    pub reserved: [u8; 3],
    /// Header size
    pub header_size: u32,
    /// Entry count
    pub entry_count: u32,
    /// Compressed block entry count
    pub compressed_block_entry_count: u32,
    /// Compressed block entry size
    pub compressed_block_entry_size: u32,
    /// Compression method name count
    pub compression_method_name_count: u32,
    /// Compression method name length
    pub compression_method_name_length: u32,
    /// Compression block size
    pub compression_block_size: u32,
    /// Directory index size
    pub directory_index_size: u32,
    /// Partition count
    pub partition_count: u32,
    /// Container id
    pub container_id: IoContainerId,
    /// Encryption key guid
    pub encryption_key_guid: Guid,
    /// Container flags
    pub container_flags: EIoContainerFlags,
    /// Reserved
    pub reserved_0: [u8; 3],
    /// Chunk perfect hash seeds count
    pub chunk_perfect_hash_seeds_count: u32,
    /// Partition size
    pub partition_size: u64,
    /// Chunks without perfect hash count
    pub chunks_without_perfect_hash_count: u32,
    /// Reserved
    pub reserved_1: [u8; 44],
}

impl IoStoreTocHeader {
    const TOC_MAGIC: [u8; 16] = *b"-==--==--==--==-";

    /// Read `IoStoreTocHeader` from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<IoStoreTocHeader, Error> {
        let mut magic = [0u8; 16];
        reader.read_exact(&mut magic)?;

        if magic != Self::TOC_MAGIC {
            return Err(IoStoreError::InvalidTocMagic(magic).into());
        }

        let version = EIoStoreTocVersion::try_from(reader.read_u8()?)?;

        let mut reserved = [0u8; 3];
        reader.read_exact(&mut reserved)?;

        let header_size = reader.read_u32::<LE>()?;

        let this_header_size = (size_of::<IoStoreTocHeader>() + Self::TOC_MAGIC.len()) as u32;
        if header_size != this_header_size {
            return Err(
                IoStoreError::invalid_toc_header_size(this_header_size, header_size).into(),
            );
        }

        let entry_count = reader.read_u32::<LE>()?;
        let compressed_block_entry_count = reader.read_u32::<LE>()?;
        let compressed_block_entry_size = reader.read_u32::<LE>()?;
        let compression_method_name_count = reader.read_u32::<LE>()?;
        let compression_method_name_length = reader.read_u32::<LE>()?;
        let compression_block_size = reader.read_u32::<LE>()?;
        let directory_index_size = reader.read_u32::<LE>()?;
        let mut partition_count = reader.read_u32::<LE>()?;
        let container_id = IoContainerId::read(reader)?;

        let encryption_key_guid = reader.read_guid()?;

        let container_flags = EIoContainerFlags::from_bits_retain(reader.read_u8()?);

        let mut reserved_0 = [0u8; 3];
        reader.read_exact(&mut reserved_0)?;

        let chunk_perfect_hash_seeds_count = reader.read_u32::<LE>()?;
        let mut partition_size = reader.read_u64::<LE>()?;
        let chunks_without_perfect_hash_count = reader.read_u32::<LE>()?;

        let mut reserved_1 = [0u8; 44];
        reader.read_exact(&mut reserved_1)?;

        if version < EIoStoreTocVersion::PartitionSize {
            partition_count = 1;
            partition_size = u64::MAX;
        }

        Ok(IoStoreTocHeader {
            version,
            reserved,
            header_size,
            entry_count,
            compressed_block_entry_count,
            compressed_block_entry_size,
            compression_method_name_count,
            compression_method_name_length,
            compression_block_size,
            directory_index_size,
            partition_count,
            container_id,
            encryption_key_guid,
            container_flags,
            reserved_0,
            chunk_perfect_hash_seeds_count,
            partition_size,
            chunks_without_perfect_hash_count,
            reserved_1,
        })
    }

    /// Write an `IoStoreTocHeader` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&Self::TOC_MAGIC)?;

        writer.write_u8(self.version as u8)?;

        writer.write_all(&self.reserved)?;

        writer.write_u32::<LE>(self.header_size)?;
        writer.write_u32::<LE>(self.entry_count)?;
        writer.write_u32::<LE>(self.compressed_block_entry_count)?;
        writer.write_u32::<LE>(self.compressed_block_entry_size)?;
        writer.write_u32::<LE>(self.compression_method_name_count)?;
        writer.write_u32::<LE>(self.compression_method_name_length)?;
        writer.write_u32::<LE>(self.compression_block_size)?;
        writer.write_u32::<LE>(self.directory_index_size)?;
        writer.write_u32::<LE>(self.partition_count)?;

        self.container_id.write(writer)?;

        writer.write_guid(&self.encryption_key_guid)?;

        writer.write_u8(self.container_flags.bits())?;

        writer.write_all(&self.reserved_0)?;

        writer.write_u32::<LE>(self.chunk_perfect_hash_seeds_count)?;
        writer.write_u64::<LE>(self.partition_size)?;
        writer.write_u32::<LE>(self.chunks_without_perfect_hash_count)?;

        writer.write_all(&self.reserved_1)?;

        Ok(())
    }
}
