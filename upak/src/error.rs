use std::fmt;
use std::io;

use crate::CompressionMethod;

#[derive(Debug)]
pub struct UpakError {
    kind: UpakErrorKind,
}

impl UpakError {
    pub fn unsupported_pak_version(version: u32) -> Self {
        UpakError {
            kind: UpakErrorKind::UnsupportedPakVersion(version),
        }
    }
    pub fn unsupported_compression(method: CompressionMethod) -> Self {
        UpakError {
            kind: UpakErrorKind::UnsupportedCompression(method),
        }
    }
    pub fn invalid_pak_file() -> Self {
        UpakError {
            kind: UpakErrorKind::InvalidPakFile,
        }
    }
    pub fn record_not_found(record_name: String) -> Self {
        UpakError {
            kind: UpakErrorKind::RecordNotFound(record_name),
        }
    }
    pub fn enrcryption_unsupported() -> Self {
        UpakError {
            kind: UpakErrorKind::EncryptionUnsupported,
        }
    }
}

impl fmt::Display for UpakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.kind {
            UpakErrorKind::UnsupportedPakVersion(ref version) => {
                format!("Unsupported pak version: {}", version)
            }
            UpakErrorKind::UnsupportedCompression(ref method) => {
                format!("Unsupported compression method: {:?}", method)
            }
            UpakErrorKind::InvalidPakFile => "Invalid pak file".to_string(),
            UpakErrorKind::RecordNotFound(ref record_name) => {
                format!("Record not found: {}", record_name)
            }
            UpakErrorKind::EncryptionUnsupported => "Encryption is not supported".to_string(),
            UpakErrorKind::IoError(ref err) => {
                format!("IO error: {}", err)
            }
        };

        write!(f, "{}", err_msg)
    }
}

impl From<io::Error> for UpakError {
    fn from(error: io::Error) -> Self {
        UpakError {
            kind: UpakErrorKind::IoError(error),
        }
    }
}

#[derive(Debug)]
pub enum UpakErrorKind {
    UnsupportedPakVersion(u32),
    UnsupportedCompression(CompressionMethod),
    InvalidPakFile,
    RecordNotFound(String),
    EncryptionUnsupported,
    IoError(io::Error),
}
