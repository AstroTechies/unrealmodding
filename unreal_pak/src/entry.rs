use std::io::{Read, Seek, SeekFrom, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

use crate::error::PakError;
use crate::header::{Block, Header};
use crate::pakversion::PakVersion;
use crate::CompressionMethod;

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
    offset: u64,
) -> Result<Vec<u8>, PakError>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(offset))?;

    let header = Header::read(reader, pak_version)?;

    match header.compression_method {
        CompressionMethod::None => {
            let mut data = vec![0u8; header.decompressed_size as usize];
            reader.read_exact(data.as_mut_slice())?;
            Ok(data)
        }
        CompressionMethod::Zlib => {
            let mut data = Vec::with_capacity(header.decompressed_size as usize);

            let compression_blocks = header
                .compression_blocks
                .as_ref()
                .ok_or_else(PakError::entry_invalid)?;
            for block in compression_blocks {
                // we do not need to seek here because the reader is at the end of the header and compression blocks are continuous
                let mut compressed_data = vec![0u8; block.size as usize];
                reader.read_exact(&mut compressed_data)?;
                let mut decoder = ZlibDecoder::new(&compressed_data[..]);
                decoder.read_to_end(&mut data)?;
            }

            Ok(data)
        }
        _ => Err(PakError::compression_unsupported(header.compression_method)),
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
    compression_method: CompressionMethod,
    block_size: u32,
) -> Result<Header, PakError>
where
    W: Write + Seek,
{
    let offset = writer.stream_position()?;
    let decompressed_size = data.len() as u64;

    // compress data in memory
    let mut compressed_data = if !matches!(compression_method, CompressionMethod::None) {
        Vec::with_capacity(data.len())
    } else {
        Vec::new()
    };
    let mut compression_blocks = None;
    let data = match compression_method {
        CompressionMethod::Zlib => {
            if pak_version < PakVersion::PakFileVersionCompressionEncryption {
                return Err(PakError::configuration_invalid());
            }

            let block_count = (data.len() as f64 / block_size as f64).ceil() as usize;
            let mut compression_blocks_inner = Vec::with_capacity(block_count);
            let header_len = Header::calculate_header_len(pak_version, Some(block_count as u32));

            for chunk in data.chunks(block_size as usize) {
                let begin = compressed_data.len() as u64;

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(chunk)?;
                let block_compressed_data = encoder.finish()?;
                compressed_data.extend_from_slice(&block_compressed_data);

                compression_blocks_inner.push(Block {
                    start: begin + header_len,
                    size: block_compressed_data.len() as u64,
                });
            }

            compression_blocks = Some(compression_blocks_inner);
            &compressed_data
        }
        CompressionMethod::None => data,
        _ => return Err(PakError::compression_unsupported(compression_method)),
    };

    let mut hasher = Sha1::new();
    hasher.update(&data);
    // sha1 always outputs 20 bytes
    let hash: [u8; 20] = hasher.finalize().to_vec().try_into().unwrap();

    let compression_block_size = if pak_version >= PakVersion::PakFileVersionCompressionEncryption {
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
        compression_method,
        hash,
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
