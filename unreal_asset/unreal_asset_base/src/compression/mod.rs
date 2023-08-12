//! Unreal decompression

use std::io::Read;

use flate2::bufread::{GzDecoder, ZlibDecoder};

use crate::error::Error;

/// Compression method
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum CompressionMethod {
    /// None
    #[default]
    None,
    /// Zlib compression
    Zlib,
    /// Gzip compression
    Gzip,
    /// Lz4 compression
    Lz4,
    /// Unknown compression format
    Unknown(Box<str>),
}

impl CompressionMethod {
    /// Create a new `CompressionMethod` from the method name
    pub fn new(name: &str) -> Self {
        match name {
            "None" => Self::None,
            "Zlib" => Self::Zlib,
            "Gzip" => Self::Gzip,
            "LZ4" => Self::Lz4,
            _ => Self::Unknown(name.to_string().into_boxed_str()),
        }
    }
}

impl ToString for CompressionMethod {
    fn to_string(&self) -> String {
        match self {
            CompressionMethod::None => String::from("None"),
            CompressionMethod::Zlib => String::from("Zlib"),
            CompressionMethod::Gzip => String::from("Gzip"),
            CompressionMethod::Lz4 => String::from("LZ4"),
            CompressionMethod::Unknown(e) => e.to_string(),
        }
    }
}

/// Decompress data with the given compression method
pub fn decompress(
    method: CompressionMethod,
    compressed: &[u8],
    decompressed: &mut [u8],
) -> Result<(), Error> {
    match method {
        CompressionMethod::None => {
            decompressed.copy_from_slice(&compressed[..decompressed.len()]);
            Ok(())
        }
        CompressionMethod::Zlib => Ok(ZlibDecoder::new(compressed).read_exact(decompressed)?),
        CompressionMethod::Gzip => Ok(GzDecoder::new(compressed).read_exact(decompressed)?),
        CompressionMethod::Lz4 => {
            lz4_flex::block::decompress_into(compressed, decompressed)?;
            Ok(())
        }
        CompressionMethod::Unknown(name) => Err(Error::UnknownCompressionMethod(name)),
    }
}
