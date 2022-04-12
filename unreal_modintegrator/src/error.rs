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
    Upak(unreal_pak::error::UpakError),
    Json(serde_json::Error),
    Integration(IntegrationError),
    Other(Box<dyn std::error::Error>),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ErrorCode::Io(ref err) => Display::fmt(err, f),
            ErrorCode::Uasset(ref err) => Display::fmt(err, f),
            ErrorCode::Upak(ref err) => Display::fmt(err, f),
            ErrorCode::Json(ref err) => Display::fmt(err, f),
            ErrorCode::Integration(ref err) => Display::fmt(err, f),
            ErrorCode::Other(ref err) => Display::fmt(err, f),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
    pub fn other(error: Box<dyn std::error::Error>) -> Self {
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

impl From<unreal_pak::error::UpakError> for Error {
    fn from(e: unreal_pak::error::UpakError) -> Self {
        Error {
            code: ErrorCode::Upak(e),
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
