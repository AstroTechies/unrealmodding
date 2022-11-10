//! Error type for unreal_pak

use std::error;
use std::fmt;
use std::io;

use crate::pakversion::PakVersion;
use crate::CompressionMethod;

/// Error type used by unreal_pak
#[derive(Debug)]
pub struct PakError {
    /// Type of the error
    pub kind: PakErrorKind,
}

impl PakError {
    /// construct UnsupportedPakVersion error
    pub fn pak_version_unsupported(version: PakVersion) -> Self {
        PakError {
            kind: PakErrorKind::PakVersionUnsupported(version),
        }
    }
    /// construct UnsupportedCompression error
    pub fn compression_unsupported(method: CompressionMethod) -> Self {
        PakError {
            kind: PakErrorKind::CompressionUnsupported(method),
        }
    }
    /// construct EncryptionUnsupported error
    pub fn enrcryption_unsupported() -> Self {
        PakError {
            kind: PakErrorKind::EncryptionUnsupported,
        }
    }
    /// construct InvalidConfiguration error
    pub fn configuration_invalid() -> Self {
        PakError {
            kind: PakErrorKind::ConfigurationInvalid,
        }
    }
    /// construct DoubleWrite error
    pub fn double_write(file_name: String) -> Self {
        PakError {
            kind: PakErrorKind::DoubleWrite(file_name),
        }
    }

    /// construct InvalidPakFile error
    pub fn pak_invalid() -> Self {
        PakError {
            kind: PakErrorKind::PakInvalid,
        }
    }
    /// construct FileNotFound error
    pub fn entry_not_found(file_name: String) -> Self {
        PakError {
            kind: PakErrorKind::EntryNotFound(file_name),
        }
    }
    /// construct InvalidFile error
    pub fn entry_invalid() -> Self {
        PakError {
            kind: PakErrorKind::EntryInvalid,
        }
    }
}

impl fmt::Display for PakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.kind {
            PakErrorKind::PakVersionUnsupported(ref version) => {
                format!("Unsupported pak version: {}", *version as u32)
            }
            PakErrorKind::CompressionUnsupported(ref method) => {
                format!("Unsupported compression method: {:?}", method)
            }
            PakErrorKind::EncryptionUnsupported => "Encryption is not supported".to_string(),
            PakErrorKind::ConfigurationInvalid => "Invalid configuration".to_string(),
            PakErrorKind::DoubleWrite(ref name) => format!(
                "Attempted to write a file twice into the same PakFile, name: {}",
                name
            ),

            PakErrorKind::PakInvalid => "Invalid pak file".to_string(),
            PakErrorKind::EntryNotFound(ref file_name) => {
                format!("File not found: {}", file_name)
            }
            PakErrorKind::EntryInvalid => "Invalid file".to_string(),

            PakErrorKind::IoError(ref err) => {
                format!("IO error: {}", err)
            }
        };

        write!(f, "{}", err_msg)
    }
}

impl From<io::Error> for PakError {
    fn from(error: io::Error) -> Self {
        PakError {
            kind: PakErrorKind::IoError(error),
        }
    }
}

impl error::Error for PakError {}

/// Error representation of PakError
#[derive(Debug)]
pub enum PakErrorKind {
    /// the pak version found is not supported by the library
    PakVersionUnsupported(PakVersion),
    /// the compression found is not supported by the library
    CompressionUnsupported(CompressionMethod),
    /// encryption is not supported
    EncryptionUnsupported,
    /// the state of a struct is invalid
    ConfigurationInvalid,
    /// Attempted to write a file twice into the same PakFile
    DoubleWrite(String),

    /// a pak file is not correctly formatted ot the file is not even a pak file
    PakInvalid,
    /// a file inside the pak file was not found
    EntryNotFound(String),
    /// a (compressed) file is corrupted or similar
    EntryInvalid,

    /// something went wrong during reading
    IoError(io::Error),
}
