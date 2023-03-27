use std::fmt::Display;
use std::io;

use unreal_pak::error::PakError;

#[derive(Debug)]
pub struct CppBootstrapperError {
    kind: CppBootstrapperErrorKind,
}

#[derive(Debug)]
pub enum CppBootstrapperErrorKind {
    Io(io::Error),
    Pak(PakError),
    Metadata(unreal_mod_metadata::error::Error),
}

impl Display for CppBootstrapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self.kind {
            CppBootstrapperErrorKind::Io(ref err) => format!("Io: {err}"),
            CppBootstrapperErrorKind::Pak(ref err) => format!("Pak: {err}"),
            CppBootstrapperErrorKind::Metadata(ref err) => format!("Metadata: {err}"),
        };

        write!(f, "{err_msg}")
    }
}

impl From<io::Error> for CppBootstrapperError {
    fn from(err: io::Error) -> Self {
        CppBootstrapperError {
            kind: CppBootstrapperErrorKind::Io(err),
        }
    }
}

impl From<PakError> for CppBootstrapperError {
    fn from(err: PakError) -> Self {
        CppBootstrapperError {
            kind: CppBootstrapperErrorKind::Pak(err),
        }
    }
}

impl From<unreal_mod_metadata::error::Error> for CppBootstrapperError {
    fn from(err: unreal_mod_metadata::error::Error) -> Self {
        CppBootstrapperError {
            kind: CppBootstrapperErrorKind::Metadata(err),
        }
    }
}

impl std::error::Error for CppBootstrapperError {}
