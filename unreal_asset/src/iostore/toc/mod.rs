//! .utoc

use std::{
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use aes::{
    cipher::{generic_array::GenericArray, KeyInit},
    Aes256,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    compression::CompressionMethod,
    error::{Error, IoStoreError},
    iostore::{
        align,
        encryption::{self, EncryptionKey, ENCRYPTION_ALIGN},
        flags::{EIoContainerFlags, IoStoreTocEntryMetaFlags},
    },
};

use self::{
    chunk::{IoChunkHash, IoChunkId},
    header::IoStoreTocHeader,
    index::IoStoreDirectoryIndex,
};

pub mod chunk;
pub mod header;
pub mod index;

/// IoStore .utoc version
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[repr(u8)]
pub enum EIoStoreTocVersion {
    /// Invalid
    Invalid = 0,
    /// Initial version
    Initial,
    /// Directory Index added
    DirectoryIndex,
    /// Partition Size added
    PartitionSize,
    /// Perfect hashing added
    PerfectHash,
    /// Perfect hashing with overflow added
    PerfectHashWithOverflow,

    /// Latest
    Latest,
    /// Latest version plus one
    LatestPlusOne,
}

/// IoStore container id
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoContainerId {
    /// Value
    pub value: u64,
}

impl IoContainerId {
    /// Read `IoContainerId` from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, Error> {
        Ok(IoContainerId {
            value: reader.read_u64::<LE>()?,
        })
    }

    /// Write `IoContainerId` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u64::<LE>(self.value)?;
        Ok(())
    }
}

/// IoStore combined offset and length
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoOffsetAndLength {
    /// Offset
    pub offset: u64,
    /// Length
    pub length: u64,
}

impl IoOffsetAndLength {
    /// Read `IoOffsetAndLength` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut value = [0u8; 10];
        reader.read_exact(&mut value)?;

        let offset = (value[4] as u64)
            | ((value[3] as u64) << 8)
            | ((value[2] as u64) << 16)
            | ((value[1] as u64) << 24)
            | ((value[0] as u64) << 32);

        let length = (value[9] as u64)
            | ((value[8] as u64) << 8)
            | ((value[7] as u64) << 16)
            | ((value[6] as u64) << 24)
            | ((value[5] as u64) << 32);

        Ok(IoOffsetAndLength { offset, length })
    }

    /// Write `IoOffsetAndLength` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut value = [0u8; 10];

        value[0] = (self.offset >> 32) as u8;
        value[1] = (self.offset >> 24) as u8;
        value[2] = (self.offset >> 16) as u8;
        value[3] = (self.offset >> 8) as u8;
        value[4] = self.offset as u8;

        value[5] = (self.length >> 32) as u8;
        value[6] = (self.length >> 24) as u8;
        value[7] = (self.length >> 16) as u8;
        value[8] = (self.length >> 8) as u8;
        value[9] = self.length as u8;

        writer.write_all(&value)?;
        Ok(())
    }
}

/// IoStore compression block entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreTocCompressedBlockEntry {
    /// Block offset
    pub offset: u64,
    /// Block compressed size
    pub compressed_size: u32,
    /// Block decompressed size
    pub decompressed_size: u32,
    /// Compresion method index
    pub compression_method_index: u8,
}

impl IoStoreTocCompressedBlockEntry {
    const OFFSET_BITS: u64 = 40;
    const OFFSET_MASK: u64 = (1u64 << Self::OFFSET_BITS).overflowing_sub(1).0;

    const SIZE_BITS: u32 = 24;
    const SIZE_MASK: u32 = (1u32 << Self::SIZE_BITS).overflowing_sub(1).0;
    const SIZE_SHIFT: u32 = 8;

    /// Read `IoStoreTocCompressedBlockEntry` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut value = [0u8; 12];
        reader.read_exact(&mut value)?;

        let offset = u64::from_le_bytes(value[..8].try_into().unwrap()) & Self::OFFSET_MASK;

        let compressed_size = (u32::from_le_bytes(value[4..8].try_into().unwrap())
            >> Self::SIZE_SHIFT)
            & Self::SIZE_MASK;
        let decompressed_size =
            u32::from_le_bytes(value[8..12].try_into().unwrap()) & Self::SIZE_MASK;

        let compression_method_index = value[11];

        Ok(IoStoreTocCompressedBlockEntry {
            offset,
            compressed_size,
            decompressed_size,
            compression_method_index,
        })
    }

    /// Write `IoStoreTocCompressedBlockEntry` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut value = [0u8; 12];

        value[..8].copy_from_slice(&(self.offset & Self::OFFSET_MASK).to_le_bytes());
        value[4..8].copy_from_slice(&(self.compressed_size << Self::SIZE_SHIFT).to_le_bytes());
        value[8..12].copy_from_slice(&(self.decompressed_size & Self::SIZE_MASK).to_le_bytes());

        value[11] = self.compression_method_index;

        writer.write_all(&value)?;
        Ok(())
    }
}

/// IoStore .utoc entry metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreTocEntryMeta {
    /// Chunk hash
    pub chunk_hash: IoChunkHash,
    /// Flags
    pub flags: IoStoreTocEntryMetaFlags,
}

impl IoStoreTocEntryMeta {
    /// Read `IoStoreTocEntryMetadata` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let chunk_hash = IoChunkHash::read(reader)?;
        let flags = IoStoreTocEntryMetaFlags::from_bits_retain(reader.read_u8()?);

        Ok(IoStoreTocEntryMeta { chunk_hash, flags })
    }

    /// Write `IoStoreTocEntryMetadata` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.chunk_hash.write(writer)?;
        writer.write_u8(self.flags.bits())?;
        Ok(())
    }
}

/// IoStore .utoc resource
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreTocResource {
    /// Header
    pub header: IoStoreTocHeader,
    /// Chunk ids
    pub chunk_ids: Vec<IoChunkId>,
    /// Chunk offsets lengths
    pub chunk_offsets_lengths: Vec<IoOffsetAndLength>,
    /// Chunk perfect hash seeds
    pub chunk_perfect_hash_seeds: Vec<i32>,
    /// Chunks without perfect hashes
    pub chunks_without_perfect_hash: Vec<u32>,
    /// Compression blocks
    pub compression_blocks: Vec<IoStoreTocCompressedBlockEntry>,
    /// Compression methods
    pub compression_methods: Vec<CompressionMethod>,
    /// Directory index
    pub directory_index: Option<IoStoreDirectoryIndex>,
    /// Chunk metas
    pub chunk_metas: Vec<IoStoreTocEntryMeta>,
}

impl IoStoreTocResource {
    /// Read `IoStoreTocResource` from a reader
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        encryption_key: Option<[u8; 32]>,
    ) -> Result<Self, Error> {
        let header = IoStoreTocHeader::read(reader)?;

        let mut chunk_ids = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            chunk_ids.push(IoChunkId::read(reader)?);
        }

        let mut chunk_offsets_lengths = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            chunk_offsets_lengths.push(IoOffsetAndLength::read(reader)?);
        }

        let mut chunk_perfect_hash_seeds = Vec::new();
        if header.version >= EIoStoreTocVersion::PerfectHash {
            for _ in 0..header.chunk_perfect_hash_seeds_count {
                chunk_perfect_hash_seeds.push(reader.read_i32::<LE>()?);
            }
        }

        let mut chunks_without_perfect_hash = Vec::new();
        if header.version >= EIoStoreTocVersion::PerfectHashWithOverflow {
            for _ in 0..header.chunks_without_perfect_hash_count {
                chunks_without_perfect_hash.push(reader.read_u32::<LE>()?);
            }
        }

        let mut compression_blocks =
            Vec::with_capacity(header.compressed_block_entry_count as usize);
        for _ in 0..header.compressed_block_entry_count {
            compression_blocks.push(IoStoreTocCompressedBlockEntry::read(reader)?);
        }

        let mut compression_methods =
            Vec::with_capacity(header.compression_method_name_count as usize);
        for _ in 0..header.compression_method_name_count {
            let mut data = vec![0u8; header.compression_method_name_length as usize];
            reader.read_exact(&mut data)?;

            let null_end = data.iter().position(|e| *e == 0x00).unwrap_or(data.len());
            data.resize(null_end, 0x00);

            compression_methods.push(CompressionMethod::new(&String::from_utf8(data)?));
        }

        if header.container_flags.contains(EIoContainerFlags::SIGNED) {
            let hash_size = reader.read_i32::<LE>()?;

            let mut toc_signature = vec![0u8; hash_size as usize];
            reader.read_exact(&mut toc_signature)?;

            let mut block_signature = vec![0u8; hash_size as usize];
            reader.read_exact(&mut block_signature)?;

            let mut joined_signatures = vec![0u8; (hash_size * 2) as usize];
            reader.read_exact(&mut joined_signatures)?;

            // skip FShaHashes
            reader.seek(SeekFrom::Current(
                (header.compressed_block_entry_count * 20) as i64,
            ))?;
        }

        let directory_index = match header.container_flags.contains(EIoContainerFlags::INDEXED)
            && header.directory_index_size > 0
        {
            true => {
                match header
                    .container_flags
                    .contains(EIoContainerFlags::ENCRYPTED)
                {
                    true => {
                        let Some(encryption_key) = encryption_key else {
                            return Err(IoStoreError::NoEncryptionKey.into());
                        };

                        let aes = Aes256::new(&GenericArray::from(encryption_key));

                        let mut buf = vec![0u8; header.directory_index_size as usize];
                        reader.read_exact(&mut buf)?;

                        encryption::decrypt(&aes, &mut buf);

                        Some(IoStoreDirectoryIndex::read(&mut Cursor::new(buf))?)
                    }
                    false => Some(IoStoreDirectoryIndex::read(reader)?),
                }
            }
            false => None,
        };

        let mut chunk_metas = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            chunk_metas.push(IoStoreTocEntryMeta::read(reader)?);
        }

        Ok(IoStoreTocResource {
            header,
            chunk_ids,
            chunk_offsets_lengths,
            chunk_perfect_hash_seeds,
            chunks_without_perfect_hash,
            compression_blocks,
            compression_methods,
            directory_index,
            chunk_metas,
        })
    }

    /// Write `IoStoreTocResource` to a writer
    pub fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        encryption_key: Option<EncryptionKey>,
    ) -> Result<(), Error> {
        let mut header = self.header.clone();

        header.container_flags.remove(EIoContainerFlags::SIGNED);
        header.container_flags.remove(EIoContainerFlags::ENCRYPTED);

        header.entry_count = self.chunk_ids.len() as u32;
        header.chunk_perfect_hash_seeds_count = self.chunk_perfect_hash_seeds.len() as u32;
        header.chunks_without_perfect_hash_count = self.chunks_without_perfect_hash.len() as u32;
        header.compressed_block_entry_count = self.compression_blocks.len() as u32;
        header.compression_method_name_count = self.compression_methods.len() as u32;
        header.directory_index_size = 0;

        let header_position = writer.stream_position()?;
        header.write(writer)?;

        for chunk_id in &self.chunk_ids {
            chunk_id.write(writer)?;
        }

        for chunk_offset_length in &self.chunk_offsets_lengths {
            chunk_offset_length.write(writer)?;
        }

        if header.version >= EIoStoreTocVersion::PerfectHash {
            for chunk_perfect_hash_seed in &self.chunk_perfect_hash_seeds {
                writer.write_i32::<LE>(*chunk_perfect_hash_seed)?;
            }
        }

        if header.version >= EIoStoreTocVersion::PerfectHashWithOverflow {
            for chunk_without_perfect_hash in &self.chunks_without_perfect_hash {
                writer.write_u32::<LE>(*chunk_without_perfect_hash)?;
            }
        }

        for compression_block in &self.compression_blocks {
            compression_block.write(writer)?;
        }

        for compression_method in &self.compression_methods {
            let mut name = compression_method.to_string().as_bytes().to_vec();
            name.resize(header.compression_method_name_length as usize, 0);

            writer.write_all(&name)?;
        }

        if let Some(ref directory_index) = self.directory_index {
            header.container_flags.insert(EIoContainerFlags::INDEXED);

            match encryption_key {
                Some(encryption_key) => {
                    header.container_flags.insert(EIoContainerFlags::ENCRYPTED);

                    let aes = Aes256::new(&GenericArray::from(encryption_key));

                    let mut aes_writer = Cursor::new(Vec::new());
                    directory_index.write(&mut aes_writer)?;

                    let mut data = aes_writer.into_inner();

                    let prev_size = data.len();
                    data.resize(
                        align::align(data.len() as u64, ENCRYPTION_ALIGN) as usize,
                        0x00,
                    );

                    for i in prev_size..data.len() {
                        data[i] = data[(i - prev_size) % data.len()];
                    }

                    encryption::encrypt(&aes, &mut data);

                    header.directory_index_size = data.len() as u32;

                    writer.write_all(&data)?;
                }
                None => {
                    directory_index.write(writer)?;
                }
            }
        }

        for chunk_meta in &self.chunk_metas {
            chunk_meta.write(writer)?;
        }

        let end_pos = writer.stream_position()?;

        writer.seek(SeekFrom::Start(header_position))?;
        header.write(writer)?;
        writer.seek(SeekFrom::Start(end_pos))?;

        Ok(())
    }

    /// Get chunk offset and length by chunk id
    pub fn get_chunk_offset(&self, id: &IoChunkId) -> Result<Option<IoOffsetAndLength>, Error> {
        let seed_index = id.hash(0)? as usize % self.chunk_perfect_hash_seeds.len();
        let seed = self.chunk_perfect_hash_seeds[seed_index];

        if seed == 0 {
            return Ok(None);
        }

        let slot = match seed < 0 {
            true => (-seed - 1) as usize,
            false => (id.hash(seed)? % self.header.entry_count as u64) as usize,
        };

        if slot >= self.chunk_ids.len() {
            return self.get_chunk_offset_imperfect(id);
        }

        if self.chunk_ids[slot] == *id {
            return Ok(Some(self.chunk_offsets_lengths[slot]));
        }

        self.get_chunk_offset_imperfect(id)
    }

    /// Get chunk offset and length by chunk type
    pub fn get_chunk_offset_by_type(
        &self,
        chunk_type: u8,
    ) -> Result<Option<IoOffsetAndLength>, Error> {
        self.get_chunk_offset(&IoChunkId::new(0, 0, chunk_type))
    }

    /// Get chunk offset and length by chunk id with imperfect hashing
    fn get_chunk_offset_imperfect(
        &self,
        id: &IoChunkId,
    ) -> Result<Option<IoOffsetAndLength>, Error> {
        Ok(self
            .chunk_ids
            .iter()
            .position(|e| e == id)
            .map(|e| self.chunk_offsets_lengths[e]))
    }
}
