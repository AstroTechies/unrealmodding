//! Compression abstraction
//! Currently supportted compressions (in addition to no compression):
//! - Zlib

//* Note: when adding more compressions you should only have to update stuff in this file, but in a few places.

use std::io::{self, Read, Seek, SeekFrom, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder};

use crate::error::PakError;
use crate::pakversion::PakVersion;

/// Enum representing which compression method is being used for an entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    /// No Compression
    #[default]
    None,
    /// Known compression method
    Known(&'static str),
    /// Unknown compression method
    Unknown([u8; 0x20]),
}

impl Compression {
    /// Create Zlib Compression configuration
    pub fn zlib() -> Self {
        Self::Known("Zlib")
    }

    pub(crate) fn from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x20];
        reader.read_exact(&mut buf)?;

        Ok(if buf == [0; 0x20] {
            Self::None
        } else if buf == pad_zeroes("Zlib".as_bytes()) {
            Self::zlib()
        } else {
            Self::Unknown(buf)
        })
    }

    pub(crate) fn from_u32(
        compression_method_num: u32,
        pak_version: PakVersion,
        compression: &CompressionMethods,
    ) -> Self {
        if pak_version >= PakVersion::FnameBasedCompressionMethod {
            if compression_method_num == 0 {
                Compression::None
            } else if compression_method_num <= 5 {
                compression.0[compression_method_num as usize - 1]
            } else {
                let mut arr = [0; 0x20];
                arr[0] = compression_method_num as u8;
                Compression::Unknown(arr)
            }
        } else {
            match compression_method_num {
                0x01 | 0x10 | 0x20 => Compression::zlib(),
                _ => Compression::None,
            }
        }
    }

    pub(crate) fn as_u32(
        &self,
        pak_version: PakVersion,
        compression: &CompressionMethods,
    ) -> Result<u32, PakError> {
        match self {
            Self::Known(method) => {
                if pak_version >= PakVersion::FnameBasedCompressionMethod {
                    match compression
                        .0
                        .iter()
                        .enumerate()
                        .find(|(_, method)| *method == self)
                    {
                        Some((i, _)) => Ok((i + 1) as u32),
                        None => Err(PakError::compression_unsupported_unknown()),
                    }
                } else {
                    match *method {
                        "Zlib" => Ok(1),
                        _ => Err(PakError::compression_unsupported_unknown()),
                    }
                }
            }
            Self::None => Ok(0),
            _ => Err(PakError::compression_unsupported_unknown()),
        }
    }

    pub(crate) fn as_bytes(&self) -> [u8; 0x20] {
        match self {
            Self::None => [0; 0x20],
            Self::Known(method) => pad_zeroes(method.as_bytes()),
            Self::Unknown(method) => *method,
        }
    }

    // These are panics becasue they should hard fail during developement.

    pub(crate) fn decompress(&self, buf: &mut Vec<u8>, data: &[u8]) -> io::Result<()> {
        match self {
            Self::Known(method) => match *method {
                "Zlib" => {
                    let mut decoder = ZlibDecoder::new(data);
                    decoder.read_to_end(buf)?;
                    Ok(())
                }
                _ => panic!("Found Compression::Known with unknown compression."),
            },
            _ => panic!("Attempted to decompress with Compression type that can't decompress."),
        }
    }

    pub(crate) fn compress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            Self::Known(method) => match *method {
                "Zlib" => {
                    let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
                    encoder.write_all(data)?;
                    Ok(encoder.finish()?)
                }
                _ => panic!("Found Compression::Known with unknown compression."),
            },
            _ => panic!("Attempted to compress with Compression type that can't compress."),
        }
    }
}

fn pad_zeroes(slice: &[u8]) -> [u8; 0x20] {
    let mut arr = [0; 0x20];
    arr[..slice.len()].copy_from_slice(slice);
    arr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CompressionMethods(pub [Compression; 5]);

impl CompressionMethods {
    pub fn zlib() -> Self {
        let mut methods = Self::default();
        methods.0[0] = Compression::zlib();
        methods
    }

    /// Read compression from provided reader. Position of the reader after return not specified.
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        // Some versions of the pak file apparently have 4 instead of 5 entries.
        // This is why first the length of the remaining stream is determined and then only
        // the existing bytes read.
        let old_pos = reader.stream_position()?;
        let remaining_len = reader.seek(SeekFrom::End(0))? - old_pos;
        reader.seek(SeekFrom::Start(old_pos))?;

        let mut methods = Self::default();

        // max 5 entries(0x20 len)
        let num_entries = 5u64.min(remaining_len / 0x20);
        for i in 0..num_entries {
            methods.0[i as usize] = Compression::from_reader(reader)?;
        }

        Ok(methods)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        // TODO UE4.22 only add 4
        let num_entries = 5usize;

        let mut buf = Vec::with_capacity(num_entries * 0x20);
        for i in 0..num_entries {
            buf.extend_from_slice(&self.0[i].as_bytes());
        }

        buf
    }
}
