use std::{error, fmt, string::FromUtf16Error};

#[derive(Debug)]
pub struct InjectorError {
    kind: InjectorErrorKind,
}

#[derive(Debug)]
pub enum InjectorErrorKind {
    Utf16(FromUtf16Error),
    WinApi(windows::core::Error),
    OutOfMemory,
}

impl fmt::Display for InjectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self.kind {
            InjectorErrorKind::Utf16(ref err) => format!("Utf16: {err}"),
            InjectorErrorKind::WinApi(ref err) => format!("WinApi: {err}"),
            InjectorErrorKind::OutOfMemory => format!("Out of memory!"),
        };

        write!(f, "{err_msg}")
    }
}

impl From<FromUtf16Error> for InjectorError {
    fn from(err: FromUtf16Error) -> Self {
        InjectorError {
            kind: InjectorErrorKind::Utf16(err),
        }
    }
}

impl From<windows::core::Error> for InjectorError {
    fn from(err: windows::core::Error) -> Self {
        InjectorError {
            kind: InjectorErrorKind::WinApi(err),
        }
    }
}

impl error::Error for InjectorError {}
