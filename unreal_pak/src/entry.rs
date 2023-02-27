use std::io::{Read, Seek, SeekFrom, Write};

use crate::compression::CompressionMethods;
use crate::error::PakError;
use crate::hash;
use crate::header::{Block, Header};
use crate::pakversion::PakVersion;
use crate::Compression;

/// Read a pak entry at the given offset in the reader
///
/// # Arguments
///
/// * `reader` - Anything that implements Read + Seek
/// * `pak_version` - Version of the pak format used
/// * `offset` - The offset of the start of the header of the file
pub(crate) fn read_entry<R>(
    reader: &mut R,
    pak_version: PakVersion,
    compression: &CompressionMethods,
    offset: u64,
) -> Result<Vec<u8>, PakError>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(offset))?;

    let header = Header::read(reader, pak_version)?;

    let compression_method = if pak_version >= PakVersion::FnameBasedCompressionMethod {
        if header.compression_method == 0 {
            Compression::None
        } else if header.compression_method <= 5 {
            compression.0[header.compression_method as usize - 1]
        } else {
            let mut arr = [0; 0x20];
            arr[0] = header.compression_method as u8;
            Compression::Unknown(arr)
        }
    } else {
        match header.compression_method {
            0x01 | 0x10 | 0x20 => Compression::zlib(),
            _ => Compression::None,
        }
    };

    match compression_method {
        Compression::None => {
            let mut data = vec![0u8; header.decompressed_size as usize];
            reader.read_exact(data.as_mut_slice())?;
            Ok(data)
        }
        Compression::Known(_) => {
            let mut data = Vec::with_capacity(header.decompressed_size as usize);

            let compression_blocks = header
                .compression_blocks
                .as_ref()
                .ok_or_else(PakError::entry_invalid)?;
            for block in compression_blocks {
                // we do not need to seek here because the reader is at the end of the header and compression blocks are continuous
                let mut compressed_data = vec![0u8; block.size as usize];
                reader.read_exact(&mut compressed_data)?;
                compression_method.decompress(&mut data, compressed_data.as_slice())?;
            }

            Ok(data)
        }
        _ => Err(PakError::compression_unsupported(compression_method)),
    }
}

/// Write an entry with Header at the position the write is at
///
/// # Arguments
///
/// * `writer` - Anything that implements Write + Seek
/// * `pak_version` - Version of the pak format to be used
/// * `data` - Uncompressed data to be written
/// * `compression_method` - What compression to use
/// * `block_size` - size of the used compression blocks
pub(crate) fn write_entry<W>(
    writer: &mut W,
    pak_version: PakVersion,
    data: &Vec<u8>,
    compress: bool,
    compression: &CompressionMethods,
    block_size: u32,
) -> Result<Header, PakError>
where
    W: Write + Seek,
{
    let offset = writer.stream_position()?;
    let decompressed_size = data.len() as u64;

    let compress = compress && decompressed_size >= 32;
    let compression_method = if compress {
        compression.0[0]
    } else {
        Compression::None
    };

    // compress data in memory
    let mut compressed_data = if compress {
        Vec::with_capacity(data.len())
    } else {
        // this will actually never be used
        Vec::new()
    };
    let mut compression_blocks = None;
    let data = match compression_method {
        Compression::Known(_) => {
            if pak_version < PakVersion::CompressionEncryption {
                return Err(PakError::configuration_invalid());
            }

            let block_count = (data.len() as f64 / block_size as f64).ceil() as usize;
            let mut compression_blocks_inner = Vec::with_capacity(block_count);
            let header_len = Header::calculate_header_len(pak_version, Some(block_count as u32));

            for chunk in data.chunks(block_size as usize) {
                let begin = compressed_data.len() as u64;

                let block_compressed_data = compression_method.compress(chunk)?;
                compressed_data.extend_from_slice(&block_compressed_data);

                compression_blocks_inner.push(Block {
                    start: begin + header_len,
                    size: block_compressed_data.len() as u64,
                });
            }

            compression_blocks = Some(compression_blocks_inner);
            &compressed_data
        }
        Compression::None => data,
        _ => return Err(PakError::compression_unsupported(compression_method)),
    };

    let compression_block_size = if pak_version >= PakVersion::CompressionEncryption {
        compression_blocks.as_ref().map(|blocks| {
            if blocks.len() == 1 {
                decompressed_size as u32
            } else {
                block_size
            }
        })
    } else {
        None
    };

    let mut header = Header {
        offset: 0x00,
        compressed_size: data.len() as u64,
        decompressed_size,
        compression_method: if compress { 1 } else { 0 },
        hash: hash(data),
        compression_blocks,
        compression_block_size,
        flags: Some(0x00),
    };

    Header::write(writer, pak_version, &header)?;
    writer.write_all(data)?;

    // the offset in the header right before the data is always 0x00, so only set here
    header.offset = offset;

    Ok(header)
}
