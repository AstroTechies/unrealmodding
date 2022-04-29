use std::error;
use std::fmt;
use std::io;

use crate::pakversion::PakVersion;
use crate::CompressionMethod;

#[derive(Debug)]
pub struct UnrealPakError {
    kind: UnrealPakErrorKind,
}

impl UnrealPakError {
    pub fn unsupported_pak_version(version: PakVersion) -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::UnsupportedPakVersion(version),
        }
    }
    pub fn unsupported_compression(method: CompressionMethod) -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::UnsupportedCompression(method),
        }
    }
    pub fn invalid_pak_file() -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::InvalidPakFile,
        }
    }
    pub fn record_not_found(record_name: String) -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::RecordNotFound(record_name),
        }
    }
    pub fn enrcryption_unsupported() -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::EncryptionUnsupported,
        }
    }
    pub fn invalid_record() -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::InvalidRecord,
        }
    }
}

impl fmt::Display for UnrealPakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.kind {
            UnrealPakErrorKind::UnsupportedPakVersion(ref version) => {
                format!("Unsupported pak version: {}", *version as u32)
            }
            UnrealPakErrorKind::UnsupportedCompression(ref method) => {
                format!("Unsupported compression method: {:?}", method)
            }
            UnrealPakErrorKind::InvalidPakFile => "Invalid pak file".to_string(),
            UnrealPakErrorKind::RecordNotFound(ref record_name) => {
                format!("Record not found: {}", record_name)
            }
            UnrealPakErrorKind::EncryptionUnsupported => "Encryption is not supported".to_string(),
            UnrealPakErrorKind::IoError(ref err) => {
                format!("IO error: {}", err)
            }
            UnrealPakErrorKind::InvalidRecord => "Invalid record".to_string(),
        };

        write!(f, "{}", err_msg)
    }
}

impl From<io::Error> for UnrealPakError {
    fn from(error: io::Error) -> Self {
        UnrealPakError {
            kind: UnrealPakErrorKind::IoError(error),
        }
    }
}

impl error::Error for UnrealPakError {}

#[derive(Debug)]
pub enum UnrealPakErrorKind {
    UnsupportedPakVersion(PakVersion),
    UnsupportedCompression(CompressionMethod),
    InvalidPakFile,
    RecordNotFound(String),
    EncryptionUnsupported,
    IoError(io::Error),
    InvalidRecord,
}
