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
            ModLoaderErrorKind::IoError(ref err) => format!("IO error: {err}"),
            ModLoaderErrorKind::IoErrorWithMessage(ref err, ref message) => {
                format!("IO error: {err}: {message}")
            }
            ModLoaderErrorKind::PakError(ref err) => format!("UnrealPak error: {err}"),
            ModLoaderErrorKind::NoBasePath => {
                "No base path found (%localappdata%\\GameName)".to_owned()
            }
            ModLoaderErrorKind::Generic(ref err) => format!("Error: {err}"),
            ModLoaderErrorKind::Other(ref msg) => format!("Other: {msg}"),
        };

        write!(f, "{err_msg}")
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
    IntegratorError(unreal_mod_integrator::error::Error),

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

    #[cfg(feature = "cpp_loader")]
    DllInjector(dll_injector::error::InjectorError),
    #[cfg(feature = "cpp_loader")]
    Json(serde_json::Error),
    #[cfg(feature = "cpp_loader")]
    CppBootstrapper(unreal_cpp_bootstrapper::error::CppBootstrapperError),

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
        let mod_name = match &self.mod_id {
            Some(mod_id) => format!("mod: {mod_id:?}, "),
            None => "".to_owned(),
        };

        let err_msg = match self.kind {
            ModLoaderWarningKind::IoError(ref err) => format!("{mod_name}IO error: {err}"),
            ModLoaderWarningKind::IoErrorWithMessage(ref err, ref message) => {
                format!("{mod_name}IO error: {err}: {message}")
            }
            ModLoaderWarningKind::UnrealPakError(ref err) => {
                format!("{mod_name}UnrealPak error: {err}")
            }
            ModLoaderWarningKind::IntegratorError(ref err) => {
                format!("{mod_name}Integrator error: {err}")
            }

            ModLoaderWarningKind::SteamError => "Failed to locate Steam installation".to_string(),
            ModLoaderWarningKind::WinStoreError => {
                "Failed to locate WinStore installation".to_string()
            }

            ModLoaderWarningKind::MissingMetadata => format!("{mod_name}Missing metadata"),
            ModLoaderWarningKind::InvalidMetadata => format!("{mod_name}Invalid metadata"),
            ModLoaderWarningKind::InvalidModId => format!("{mod_name}Invalid mod ID"),
            ModLoaderWarningKind::InvalidModFileName => {
                format!("{mod_name}Invalid mod file name")
            }
            ModLoaderWarningKind::InvalidVersion => format!("{mod_name}Invalid version"),
            ModLoaderWarningKind::IndexFileDownloadFailed(ref err) => {
                format!("{mod_name}Failed to download index file {err}")
            }
            ModLoaderWarningKind::IndexFileDownloadFailedStatus(ref status) => {
                format!("{mod_name}Failed to download index file, status: {status}")
            }
            ModLoaderWarningKind::InvalidIndexFile => format!("{mod_name}Invalid index file"),
            ModLoaderWarningKind::IndexFileMissingMod => {
                format!("{mod_name}Index file missing mod")
            }
            ModLoaderWarningKind::DownloadFailed(ref err) => {
                format!("{mod_name}Download failed: {err}")
            }

            #[cfg(feature = "cpp_loader")]
            ModLoaderWarningKind::DllInjector(ref err) => format!("Injector: {err}"),
            #[cfg(feature = "cpp_loader")]
            ModLoaderWarningKind::Json(ref err) => format!("Json: {err}"),
            #[cfg(feature = "cpp_loader")]
            ModLoaderWarningKind::CppBootstrapper(ref err) => format!("Cpp boostrapper: {err}"),

            ModLoaderWarningKind::Other(ref message) => format!("{mod_name}{message}"),
            ModLoaderWarningKind::Generic(ref err) => format!("{mod_name}Error: {err}"),
            ModLoaderWarningKind::UnresolvedDependency(ref dependency, ref requesters) => {
                format!(
                    "Error: Unresolved dependency {} for mods: \n{}",
                    dependency,
                    requesters
                        .iter()
                        .map(|(requester, requested_version)| format!(
                            "{requester}: {requested_version}\n"
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

        write!(f, "{err_msg}")
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

impl From<unreal_mod_integrator::error::Error> for ModLoaderWarning {
    fn from(err: unreal_mod_integrator::error::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::IntegratorError(err),
            mod_id: None,
        }
    }
}

#[cfg(feature = "cpp_loader")]
impl From<dll_injector::error::InjectorError> for ModLoaderWarning {
    fn from(err: dll_injector::error::InjectorError) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::DllInjector(err),
            mod_id: None,
        }
    }
}

#[cfg(feature = "cpp_loader")]
impl From<serde_json::Error> for ModLoaderWarning {
    fn from(err: serde_json::Error) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::Json(err),
            mod_id: None,
        }
    }
}

#[cfg(feature = "cpp_loader")]
impl From<unreal_cpp_bootstrapper::error::CppBootstrapperError> for ModLoaderWarning {
    fn from(err: unreal_cpp_bootstrapper::error::CppBootstrapperError) -> Self {
        ModLoaderWarning {
            kind: ModLoaderWarningKind::CppBootstrapper(err),
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
