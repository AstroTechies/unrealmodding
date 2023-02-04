use std::error;
use std::fmt;
use std::string::FromUtf16Error;

#[derive(Debug)]
pub struct InjectorError {
    kind: InjectorErrorKind,
}

#[derive(Debug)]
pub enum InjectorErrorKind {
    Utf16(FromUtf16Error),
    #[cfg(windows)]
    WinApi(windows::core::Error),
    OutOfMemory,
}

impl InjectorError {
    pub fn out_of_memory() -> Self {
        InjectorError {
            kind: InjectorErrorKind::OutOfMemory,
        }
    }
}

impl fmt::Display for InjectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self.kind {
            InjectorErrorKind::Utf16(ref err) => format!("Utf16: {err}"),
            #[cfg(windows)]
            InjectorErrorKind::WinApi(ref err) => format!("WinApi: {err}"),
            InjectorErrorKind::OutOfMemory => "Out of memory!".to_string(),
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

#[cfg(windows)]
impl From<windows::core::Error> for InjectorError {
    fn from(err: windows::core::Error) -> Self {
        InjectorError {
            kind: InjectorErrorKind::WinApi(err),
        }
    }
}

impl error::Error for InjectorError {}
