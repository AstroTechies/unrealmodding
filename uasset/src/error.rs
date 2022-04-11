use std::fmt::Formatter;
use std::string::{FromUtf16Error, FromUtf8Error};
use std::{error, fmt::Display, io};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug)]
pub enum KismetError {
    InvalidToken(Box<str>),
    UnknownExpression(Box<str>),
}

impl KismetError {
    pub fn token(msg: String) -> Self {
        KismetError::InvalidToken(msg.into_boxed_str()) // todo: maybe not allocate a string
    }

    pub fn expression(msg: String) -> Self {
        KismetError::UnknownExpression(msg.into_boxed_str())
    }
}

impl Display for KismetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            KismetError::InvalidToken(ref err) => f.write_str(err),
            KismetError::UnknownExpression(ref err) => f.write_str(err),
        }
    }
}

#[derive(Debug)]
pub enum PropertyError {
    HeaderlessProperty,
    PropertyFieldNone(Box<str>, Box<str>),
    InvalidStruct(Box<str>),
    InvalidArrayType(Box<str>),
    Other(Box<str>),
}

impl PropertyError {
    pub fn headerless() -> Self {
        PropertyError::HeaderlessProperty
    }

    pub fn property_field_none(field_name: &str, expected: &str) -> Self {
        PropertyError::PropertyFieldNone(
            field_name.to_string().into_boxed_str(),
            expected.to_string().into_boxed_str(),
        )
    }

    pub fn invalid_struct(msg: String) -> Self {
        PropertyError::InvalidStruct(msg.into_boxed_str())
    }

    pub fn invalid_array(msg: String) -> Self {
        PropertyError::InvalidArrayType(msg.into_boxed_str())
    }

    pub fn other(msg: String) -> Self {
        PropertyError::Other(msg.into_boxed_str())
    }
}

impl Display for PropertyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            PropertyError::HeaderlessProperty => {
                write!(f, "include_header: true on headerless property")
            }
            PropertyError::PropertyFieldNone(ref field_name, ref expected) => {
                write!(f, "{} is None expected {}", field_name, expected)
            }
            PropertyError::InvalidStruct(ref err) => f.write_str(err),
            PropertyError::InvalidArrayType(ref err) => f.write_str(err),
            PropertyError::Other(ref err) => f.write_str(err),
        }
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Utf16(FromUtf16Error),
    NoData(Box<str>),
    InvalidFile(Box<str>),
    InvalidPackageIndex(Box<str>),
    InvalidEnumValue(Box<str>),
    Unimplemented(Box<str>),
    Kismet(KismetError),
    Property(PropertyError),
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
    pub fn no_data(msg: String) -> Self {
        Error {
            code: ErrorCode::NoData(msg.into_boxed_str()),
        }
    }

    pub fn invalid_file(msg: String) -> Self {
        Error {
            code: ErrorCode::InvalidFile(msg.into_boxed_str()),
        }
    }

    pub fn invalid_package_index(msg: String) -> Self {
        Error {
            code: ErrorCode::InvalidPackageIndex(msg.into_boxed_str()),
        }
    }

    pub fn unimplemented(msg: String) -> Self {
        Error {
            code: ErrorCode::Unimplemented(msg.into_boxed_str()),
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

//tmp
impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, "O")
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error {
            code: ErrorCode::Utf8(e),
        }
    }
}

impl From<FromUtf16Error> for Error {
    fn from(e: FromUtf16Error) -> Self {
        Error {
            code: ErrorCode::Utf16(e),
        }
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(e: TryFromPrimitiveError<T>) -> Self {
        Error {
            code: ErrorCode::InvalidEnumValue(e.to_string().into_boxed_str()),
        }
    }
}

impl From<KismetError> for Error {
    fn from(e: KismetError) -> Self {
        Error {
            code: ErrorCode::Kismet(e),
        }
    }
}

impl From<PropertyError> for Error {
    fn from(e: PropertyError) -> Self {
        Error {
            code: ErrorCode::Property(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.code, f)
    }
}

impl error::Error for Error {}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ErrorCode::Io(ref err) => Display::fmt(err, f),
            ErrorCode::Utf8(ref err) => Display::fmt(err, f),
            ErrorCode::Utf16(ref err) => Display::fmt(err, f),
            ErrorCode::NoData(ref err) => Display::fmt(err, f),
            ErrorCode::InvalidFile(ref err) => f.write_str(err),
            ErrorCode::InvalidPackageIndex(ref err) => f.write_str(err),
            ErrorCode::InvalidEnumValue(ref err) => f.write_str(err),
            ErrorCode::Unimplemented(ref err) => f.write_str(err),
            ErrorCode::Kismet(ref err) => Display::fmt(err, f),
            ErrorCode::Property(ref err) => Display::fmt(err, f),
        }
    }
}
