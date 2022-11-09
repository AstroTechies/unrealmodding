/*
    legacy/entry header (version <= 9):
    - u64 offset (when infront of file empty)
    - u64 size
    - u64 size decompressed
    - u32 compression method
    - 20 bytes sha1 hash
    - compression block data (only when compression method is not 0)
        - u32 number of blocks
        - blocks
            - u64 block start
            - u64 block end
    - u8 is encrypted flag
    - u32 block size
*/

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;

use crate::error::PakError;
use crate::pakversion::PakVersion;
use crate::CompressionMethod;

#[derive(Debug)]
pub(crate) struct Header {
    /// This may incorrectly be 0x00
    pub offset: u64,
    pub compressed_size: u64,
    pub decompressed_size: u64,
    pub compression_method: CompressionMethod,
    pub hash: [u8; 20],
    pub compression_blocks: Option<Vec<Block>>,
    pub flags: Option<u8>,
    pub compression_block_size: Option<u32>,
}

/// One compression block
#[derive(Debug, Clone)]
pub(crate) struct Block {
    /// Start offset relative to the start of the header of the entry
    pub start: u64,
    /// size of the compressed block
    pub size: u64,
}

impl Header {
    /// Read data from the reader into a Header, reader needs to be set at start of a header
    pub(crate) fn read<R: Read>(reader: &mut R, pak_version: PakVersion) -> Result<Self, PakError> {
        let offset = reader.read_u64::<LittleEndian>()?;

        let compressed_size = reader.read_u64::<LittleEndian>()?;
        let decompressed_size = reader.read_u64::<LittleEndian>()?;

        // TODO: the assumption of 0x01 = zlib is actually incorrect after PakFileVersionFnameBasedCompressionMethod,
        // it is actually an index into the compression format name array in the footer
        let compression_method =
            match CompressionMethod::try_from_primitive(reader.read_i32::<LittleEndian>()?) {
                Ok(compression_method) => compression_method,
                Err(_) => CompressionMethod::Unknown,
            };

        if pak_version <= PakVersion::PakFileVersionInitial {
            let _timestamp = reader.read_u64::<LittleEndian>()?;
        }

        let mut hash = [0u8; 20];
        reader.read_exact(&mut hash)?;

        let mut compression_blocks = None;
        let mut flags = None;
        let mut compression_block_size = None;

        if pak_version >= PakVersion::PakFileVersionCompressionEncryption {
            if compression_method != CompressionMethod::None {
                let block_count = reader.read_u32::<LittleEndian>()? as usize;
                let mut compression_blocks_inner = Vec::with_capacity(block_count);

                for _ in 0..block_count {
                    // convert old absolute to relative offsets
                    let start_offset = reader.read_u64::<LittleEndian>()?
                        - if pak_version < PakVersion::PakFileVersionRelativeChunkOffsets {
                            offset
                        } else {
                            0
                        };
                    let end_offset = reader.read_u64::<LittleEndian>()?;
                    compression_blocks_inner.push(Block {
                        start: start_offset,
                        size: end_offset - start_offset,
                    });
                }
                compression_blocks = Some(compression_blocks_inner);
            }

            flags = Some(reader.read_u8()?);
            compression_block_size = Some(reader.read_u32::<LittleEndian>()?);
        }

        Ok(Header {
            offset,
            compressed_size,
            decompressed_size,
            compression_method,
            hash,
            compression_blocks,
            compression_block_size,
            flags,
        })
    }

    /// Write data from a Header into the writer, writer needs to be set where the header is supposed to be written
    pub(crate) fn write<W: Write>(
        writer: &mut W,
        pak_version: PakVersion,
        header: &Self,
    ) -> Result<(), PakError> {
        writer.write_u64::<LittleEndian>(header.offset)?;
        writer.write_u64::<LittleEndian>(header.compressed_size)?;
        writer.write_u64::<LittleEndian>(header.decompressed_size)?;
        writer.write_i32::<LittleEndian>(header.compression_method.into())?;

        writer.write_all(&header.hash)?;

        if pak_version >= PakVersion::PakFileVersionCompressionEncryption {
            if header.compression_method != CompressionMethod::None {
                if let Some(compression_blocks) = &header.compression_blocks {
                    writer.write_u32::<LittleEndian>(compression_blocks.len() as u32)?;
                    for block in compression_blocks {
                        writer.write_u64::<LittleEndian>(block.start)?;
                        writer.write_u64::<LittleEndian>(block.start + block.size)?;
                    }
                }
            }

            writer.write_u8(header.flags.unwrap_or(0))?;
            writer.write_u32::<LittleEndian>(header.compression_block_size.unwrap_or(0x010000))?;
        }

        Ok(())
    }

    pub(crate) fn calculate_header_len(pak_version: PakVersion, block_count: Option<u32>) -> u64 {
        let mut len = 0;

        // offset, size, decomp size, comp method
        len += 28;

        // timestamp
        if pak_version <= PakVersion::PakFileVersionInitial {
            len += 8;
        }

        // hash
        len += 20;

        if pak_version >= PakVersion::PakFileVersionCompressionEncryption {
            if let Some(block_count) = block_count {
                // block count
                len += 4;

                // block start, end
                len += block_count as u64 * 16;
            }

            // flags (u8) + compression block size
            len += 5;
        }

        len
    }
}
