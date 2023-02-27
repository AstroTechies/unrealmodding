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

use std::io::{self, Read, Seek, Write};

use bitvec::prelude::*;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::error::PakError;
use crate::pakversion::PakVersion;

#[derive(Debug)]
pub(crate) struct Header {
    /// This may incorrectly be 0x00
    pub offset: u64,
    pub compressed_size: u64,
    pub decompressed_size: u64,
    pub compression_method: u32,
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
        let offset = reader.read_u64::<LE>()?;

        let compressed_size = reader.read_u64::<LE>()?;
        let decompressed_size = reader.read_u64::<LE>()?;

        // TODO UE4.22 apparently this can be 1 byte too?
        let compression_method = reader.read_u32::<LE>()?;

        if pak_version <= PakVersion::Initial {
            let _timestamp = reader.read_u64::<LE>()?;
        }

        let mut hash = [0u8; 20];
        reader.read_exact(&mut hash)?;

        let mut compression_blocks = None;
        let mut flags = None;
        let mut compression_block_size = None;

        if pak_version >= PakVersion::CompressionEncryption {
            if compression_method != 0 {
                let block_count = reader.read_u32::<LE>()? as usize;
                let mut compression_blocks_inner = Vec::with_capacity(block_count);

                for _ in 0..block_count {
                    // convert old absolute to relative offsets
                    let start_offset = reader.read_u64::<LE>()?
                        - if pak_version < PakVersion::RelativeChunkOffsets {
                            offset
                        } else {
                            0
                        };
                    let end_offset = reader.read_u64::<LE>()?;
                    compression_blocks_inner.push(Block {
                        start: start_offset,
                        size: end_offset - start_offset,
                    });
                }
                compression_blocks = Some(compression_blocks_inner);
            }

            flags = Some(reader.read_u8()?);
            compression_block_size = Some(reader.read_u32::<LE>()?);
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

    /// Read (bit)encoded header
    pub(crate) fn read_encoded<R: Read + Seek>(
        reader: &mut R,
        _pak_version: PakVersion,
    ) -> Result<Self, PakError> {
        let mut header_bits = [0u8; 4];
        reader.read_exact(&mut header_bits)?;
        let header_bits = header_bits.view_bits::<Lsb0>();

        // compression blocks
        // this is actually irrelevant here because it is also included in the header before the actual data
        let mut block_size = header_bits[0..=5].load_le::<u32>();
        let _block_count = header_bits[6..=21].load_le::<u32>();

        if block_size == 0x3f {
            block_size = reader.read_u32::<LE>()?;
        } else {
            block_size <<= 11;
        }

        let is_encrypted = header_bits[22];
        let compression_method = header_bits[23..=28].load_le::<u32>();

        let mut read_size = |bit: usize| -> io::Result<_> {
            Ok(if header_bits[bit] {
                reader.read_u32::<LE>()? as u64
            } else {
                reader.read_u64::<LE>()?
            })
        };

        let offset = read_size(31)?;
        let decompressed_size = read_size(30)?;
        let compressed_size = if compression_method == 0 {
            decompressed_size
        } else {
            read_size(29)?
        };

        // compression blocks could be read here but why waste the time, they won't get used anyways

        Ok(Header {
            offset,
            compressed_size,
            decompressed_size,
            compression_method,
            hash: [0; 20],
            compression_blocks: None,
            compression_block_size: Some(block_size),
            flags: Some(if is_encrypted { 1 } else { 0 }),
        })
    }

    /// Write data from a Header into the writer, writer needs to be set where the header is supposed to be written
    pub(crate) fn write<W: Write>(
        writer: &mut W,
        pak_version: PakVersion,
        header: &Self,
    ) -> Result<(), PakError> {
        writer.write_u64::<LE>(header.offset)?;
        writer.write_u64::<LE>(header.compressed_size)?;
        writer.write_u64::<LE>(header.decompressed_size)?;
        writer.write_u32::<LE>(header.compression_method)?;

        writer.write_all(&header.hash)?;

        if pak_version >= PakVersion::CompressionEncryption {
            if header.compression_method != 0 {
                if let Some(compression_blocks) = &header.compression_blocks {
                    writer.write_u32::<LE>(compression_blocks.len() as u32)?;
                    for block in compression_blocks {
                        writer.write_u64::<LE>(block.start)?;
                        writer.write_u64::<LE>(block.start + block.size)?;
                    }
                }
            }

            writer.write_u8(header.flags.unwrap_or(0))?;
            writer.write_u32::<LE>(header.compression_block_size.unwrap_or(0x010000))?;
        }

        Ok(())
    }

    pub(crate) fn calculate_header_len(pak_version: PakVersion, block_count: Option<u32>) -> u64 {
        let mut len = 0;

        // offset, size, decomp size, comp method
        len += 28;

        // timestamp
        if pak_version <= PakVersion::Initial {
            len += 8;
        }

        // hash
        len += 20;

        if pak_version >= PakVersion::CompressionEncryption {
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
