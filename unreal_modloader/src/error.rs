use std::error;
use std::fmt;
use std::io;

use reqwest::StatusCode;
use unreal_pak::error::PakError;

/// For critical errors that can happen during runtime which prevent further
/// operation of the modloader and cannot be handled gracefully.
#[derive(Debug)]
pub struct ModLoaderError {
    pub kind: ModLoaderErrorKind,
}

#[derive(Debug)]
pub enum ModLoaderErrorKind {
    IoError(io::Error),
    IoErrorWithMessage(io::Error, String),
    PakError(PakError),
    NoBasePath,
    Generic(Box<dyn std::error::Error + Send>),
    Other(Box<str>),
}

impl ModLoaderError {
    pub fn io_error_with_message(message: String, err: io::Error) -> Self {
        ModLoaderError {
            kind: ModLoaderErrorKind::IoErrorWithMessage(err, message),
        }
    }
    pub fn no_base_path() -> Self {
        ModLoaderError {
            kind: ModLoaderErrorKind::NoBasePath,
        }
    }

    pub fn other(msg: String) -> Self {
        ModLoaderError {
            kind: ModLoaderErrorKind::Other(msg.into_boxed_str()),
        }
    }
}

impl fmt::Display for ModLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.kind {
            ModLoaderErrorKind::IoError(ref err) => format!("IO error: {}", err),
            ModLoaderErrorKind::IoErrorWithMessage(ref err, ref message) => {
                format!("IO error: {}: {}", err, message)
            }
            ModLoaderErrorKind::PakError(ref err) => format!("UnrealPak error: {}", err),
            ModLoaderErrorKind::NoBasePath => {
                "No base path found (%localappdata%\\GameName)".to_owned()
            }
            ModLoaderErrorKind::Generic(ref err) => format!("Error: {}", err),
            ModLoaderErrorKind::Other(ref msg) => format!("Other: {}", msg),
        };

        write!(f, "{}", err_msg)
    }
}

impl From<io::Error> for ModLoaderError {
    fn from(err: io::Error) -> Self {
        ModLoaderError {
            kind: ModLoaderErrorKind::IoError(err),
        }
    }
}

impl From<PakError> for ModLoaderError {
    fn from(err: PakError) -> Self {
        ModLoaderError {
            kind: ModLoaderErrorKind::PakError(err),
        }
    }
}

impl error::Error for ModLoaderError {}

/// For non-critical errors that can happen during runtime which can be
/// handled gracefully. These often occur on a per mod basis and will simply
/// be displayed to the user.
#[derive(Debug)]
pub struct ModLoaderWarning {
    /// type of warning
    pub kind: ModLoaderWarningKind,
    /// identifier of the mod that this warning is related to,
    /// can be filename or mod_id
    pub mod_id: Option<String>,
}

#[derive(Debug)]
pub enum ModLoaderWarningKind {
    IoError(io::Error),
    IoErrorWithMessage(io::Error, String),
    UnrealPakError(PakError),
    IntegratorError(unreal_modintegrator::error::Error),

    UnresolvedDependency(String, Vec<(String, String)>),
    ReferencedByOtherMods(String, Vec<String>),

    SteamError,
    WinStoreError,

    MissingMetadata,
    InvalidMetadata,
    InvalidModId,
    InvalidModFileName,
    InvalidVersion,
    IndexFileDownloadFailed(reqwest::Error),
    IndexFileDownloadFailedStatus(StatusCode),
    InvalidIndexFile,
    IndexFileMissingMod,
    DownloadFailed(reqwest::Error),

    Other(String),
    Generic(Box<dyn std::error::Error + Send>),
}

impl ModLoaderWarning {
    pub fn with_mod_id(mut self, mod_id: String) -> Self {
        self.mod_id = Some(mod_id);
        self
    }

    pub fn io_error_with_message(message: String, err: io::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IoErrorWithMessage(err, message),
            mod_id: None,
        }
    }

    pub fn unresolved_dependency(mod_id: String, requesting_modids: Vec<(String, String)>) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::UnresolvedDependency(mod_id, requesting_modids),
            mod_id: None,
        }
    }

    pub fn referenced_by_other_mods(mod_id: String, referencers: Vec<String>) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::ReferencedByOtherMods(mod_id.clone(), referencers),
            mod_id: Some(mod_id),
        }
    }

    pub fn steam_error() -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::SteamError,
            mod_id: None,
        }
    }
    pub fn winstore_error() -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::WinStoreError,
            mod_id: None,
        }
    }
    pub fn missing_metadata(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::MissingMetadata,
            mod_id: Some(mod_id),
        }
    }
    pub fn invalid_metadata(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::InvalidMetadata,
            mod_id: Some(mod_id),
        }
    }
    pub fn invalid_mod_id(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::InvalidModId,
            mod_id: Some(mod_id),
        }
    }
    pub fn invalid_mod_file_name(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::InvalidModFileName,
            mod_id: Some(mod_id),
        }
    }
    pub fn invalid_version(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::InvalidVersion,
            mod_id: Some(mod_id),
        }
    }
    pub fn index_file_download_failed(mod_id: String, err: reqwest::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IndexFileDownloadFailed(err),
            mod_id: Some(mod_id),
        }
    }
    pub fn index_file_download_failed_status(mod_id: String, status: StatusCode) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IndexFileDownloadFailedStatus(status),
            mod_id: Some(mod_id),
        }
    }
    pub fn invalid_index_file(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::InvalidIndexFile,
            mod_id: Some(mod_id),
        }
    }
    pub fn index_file_missing_mod(mod_id: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IndexFileMissingMod,
            mod_id: Some(mod_id),
        }
    }
    pub fn download_failed(mod_id: String, err: reqwest::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::DownloadFailed(err),
            mod_id: Some(mod_id),
        }
    }

    pub fn other(message: String) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::Other(message),
            mod_id: None,
        }
    }
}

impl fmt::Display for ModLoaderWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mod_name = if self.mod_id.is_some() {
            format!("mod: {:?}, ", self.mod_id.as_ref().unwrap())
        } else {
            "".to_owned()
        };

        let err_msg = match self.kind {
            ModLoaderWarningKind::IoError(ref err) => format!("{}IO error: {}", mod_name, err),
            ModLoaderWarningKind::IoErrorWithMessage(ref err, ref message) => {
                format!("{}IO error: {}: {}", mod_name, err, message)
            }
            ModLoaderWarningKind::UnrealPakError(ref err) => {
                format!("{}UnrealPak error: {}", mod_name, err)
            }
            ModLoaderWarningKind::IntegratorError(ref err) => {
                format!("{}Integrator error: {}", mod_name, err)
            }

            ModLoaderWarningKind::SteamError => "Failed to locate Steam installation".to_string(),
            ModLoaderWarningKind::WinStoreError => {
                "Failed to locate WinStore installation".to_string()
            }

            ModLoaderWarningKind::MissingMetadata => format!("{}Missing metadata", mod_name),
            ModLoaderWarningKind::InvalidMetadata => format!("{}Invalid metadata", mod_name),
            ModLoaderWarningKind::InvalidModId => format!("{}Invalid mod ID", mod_name),
            ModLoaderWarningKind::InvalidModFileName => {
                format!("{}Invalid mod file name", mod_name)
            }
            ModLoaderWarningKind::InvalidVersion => format!("{}Invalid version", mod_name),
            ModLoaderWarningKind::IndexFileDownloadFailed(ref err) => {
                format!("{}Failed to download index file {}", mod_name, err)
            }
            ModLoaderWarningKind::IndexFileDownloadFailedStatus(ref status) => format!(
                "{}Failed to download index file, status: {}",
                mod_name, status
            ),
            ModLoaderWarningKind::InvalidIndexFile => format!("{}Invalid index file", mod_name),
            ModLoaderWarningKind::IndexFileMissingMod => {
                format!("{}Index file missing mod", mod_name)
            }
            ModLoaderWarningKind::DownloadFailed(ref err) => {
                format!("{}Download failed: {}", mod_name, err)
            }

            ModLoaderWarningKind::Other(ref message) => format!("{}{}", mod_name, message),
            ModLoaderWarningKind::Generic(ref err) => format!("{}Error: {}", mod_name, err),
            ModLoaderWarningKind::UnresolvedDependency(ref dependency, ref requesters) => {
                format!(
                    "Error: Unresolved dependency {} for mods: \n{}",
                    dependency,
                    requesters
                        .iter()
                        .map(|(requester, requested_version)| format!(
                            "{}: {}\n",
                            requester, requested_version
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            ModLoaderWarningKind::ReferencedByOtherMods(ref mod_id, ref referencers) => format!(
                "Error: {} is required by these mods, disable them to disable this mod: \n{}",
                mod_id,
                referencers.join("\n")
            ),
        };

        write!(f, "{}", err_msg)
    }
}

impl From<io::Error> for ModLoaderWarning {
    fn from(err: io::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IoError(err),
            mod_id: None,
        }
    }
}

impl From<PakError> for ModLoaderWarning {
    fn from(err: PakError) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::UnrealPakError(err),
            mod_id: None,
        }
    }
}

impl From<unreal_modintegrator::error::Error> for ModLoaderWarning {
    fn from(err: unreal_modintegrator::error::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IntegratorError(err),
            mod_id: None,
        }
    }
}

// impl From<Box<dyn std::error::Error + Send>> for ModLoaderWarning {
//     fn from(err: Box<dyn std::error::Error + Send>) -> Self {
//         ModLoaderWarning {
//             kind: ModLoaderWarningKind::Generic(err),
//             mod_id: None,
//         }
//     }
// }

impl error::Error for ModLoaderWarning {}
