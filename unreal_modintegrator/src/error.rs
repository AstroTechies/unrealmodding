use std::{fmt::Display, io};

#[derive(Debug)]
pub enum IntegrationError {
    GameNotFound,
    CorruptedStarterPak,
}

impl IntegrationError {
    pub fn game_not_found() -> Self {
        Self::GameNotFound
    }

    pub fn corrupted_starter_pak() -> Self {
        Self::CorruptedStarterPak
    }
}

impl Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IntegrationError::GameNotFound => write!(f, "Game not found"),
            IntegrationError::CorruptedStarterPak => write!(f, "Corrupted starter pak"),
        }
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    Io(io::Error),
    Uasset(unreal_asset::error::Error),
    UnrealPak(unreal_pak::error::UnrealPakError),
    UnrealModMetaData(unreal_modmetadata::error::Error),
    Json(serde_json::Error),
    Integration(IntegrationError),
    Other(Box<dyn std::error::Error + Send>),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ErrorCode::Io(ref err) => Display::fmt(err, f),
            ErrorCode::Uasset(ref err) => Display::fmt(err, f),
            ErrorCode::UnrealPak(ref err) => Display::fmt(err, f),
            ErrorCode::Json(ref err) => Display::fmt(err, f),
            ErrorCode::Integration(ref err) => Display::fmt(err, f),
            ErrorCode::Other(ref err) => Display::fmt(err, f),
            ErrorCode::UnrealModMetaData(ref err) => Display::fmt(err, f),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
    pub fn other(error: Box<dyn std::error::Error + Send>) -> Self {
        Error {
            code: ErrorCode::Other(error),
        }
    }
}

impl From<IntegrationError> for Error {
    fn from(e: IntegrationError) -> Self {
        Error {
            code: ErrorCode::Integration(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            code: ErrorCode::Io(e),
        }
    }
}

impl From<unreal_asset::error::Error> for Error {
    fn from(e: unreal_asset::error::Error) -> Self {
        Error {
            code: ErrorCode::Uasset(e),
        }
    }
}

impl From<unreal_pak::error::UnrealPakError> for Error {
    fn from(e: unreal_pak::error::UnrealPakError) -> Self {
        Error {
            code: ErrorCode::UnrealPak(e),
        }
    }
}

impl From<unreal_modmetadata::error::Error> for Error {
    fn from(e: unreal_modmetadata::error::Error) -> Self {
        Error {
            code: ErrorCode::UnrealModMetaData(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error {
            code: ErrorCode::Json(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.code, f)
    }
}
