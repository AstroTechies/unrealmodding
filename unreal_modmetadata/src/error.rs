use std::fmt::Display;

#[derive(Debug)]
pub enum ErrorCode {
    InvalidMetadata,
    UnsupportedSchema(u64),

    Json(serde_json::Error),
    SemVer(semver::Error),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ErrorCode::InvalidMetadata => f.write_str("Invalid metadata"),
            ErrorCode::UnsupportedSchema(schema) => {
                write!(f, "Unsupported schema version {}", schema)
            }
            ErrorCode::Json(ref err) => Display::fmt(err, f),
            ErrorCode::SemVer(ref err) => Display::fmt(err, f),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
    pub fn invalid_metadata() -> Self {
        Error {
            code: ErrorCode::InvalidMetadata,
        }
    }

    pub fn unsupported_schema(schema: u64) -> Self {
        Error {
            code: ErrorCode::UnsupportedSchema(schema),
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

impl From<semver::Error> for Error {
    fn from(e: semver::Error) -> Self {
        Error {
            code: ErrorCode::SemVer(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.code, f)
    }
}
