use std::convert;
use std::error;
use std::fmt;
use std::num::ParseIntError;

/// Version of a mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
pub struct VersionConvertError;

impl error::Error for VersionConvertError {}
impl fmt::Display for VersionConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VersionConvertError")
    }
}
impl convert::From<ParseIntError> for VersionConvertError {
    fn from(_: ParseIntError) -> Self {
        VersionConvertError
    }
}

impl TryFrom<&String> for Version {
    type Error = VersionConvertError;

    fn try_from(s: &String) -> Result<Self, VersionConvertError> {
        let mut parts = s.split('.');
        let major = parts.next().ok_or(VersionConvertError)?.parse::<usize>()?;
        let minor = parts.next().ok_or(VersionConvertError)?.parse::<usize>()?;
        let patch = parts.next().ok_or(VersionConvertError)?.parse::<usize>()?;
        Ok(Self::new(major, minor, patch))
    }
}

/// Version of the Unreal Engine game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameBuild {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
    pub build: usize,
}

impl GameBuild {
    pub fn new(major: usize, minor: usize, patch: usize, build: usize) -> Self {
        GameBuild {
            major,
            minor,
            patch,
            build,
        }
    }
}

impl fmt::Display for GameBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.patch, self.build
        )
    }
}

#[derive(Debug)]
pub struct GameBuildConvertError;

impl error::Error for GameBuildConvertError {}
impl fmt::Display for GameBuildConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameBuildConvertError")
    }
}
impl convert::From<ParseIntError> for GameBuildConvertError {
    fn from(_: ParseIntError) -> Self {
        GameBuildConvertError
    }
}

impl TryFrom<&String> for GameBuild {
    type Error = GameBuildConvertError;

    fn try_from(s: &String) -> Result<Self, GameBuildConvertError> {
        let mut parts = s.split('.');
        let major = parts
            .next()
            .ok_or(GameBuildConvertError)?
            .parse::<usize>()?;
        let minor = parts
            .next()
            .ok_or(GameBuildConvertError)?
            .parse::<usize>()?;
        let patch = parts
            .next()
            .ok_or(GameBuildConvertError)?
            .parse::<usize>()?;
        let build = parts
            .next()
            .ok_or(GameBuildConvertError)?
            .parse::<usize>()?;
        Ok(Self::new(major, minor, patch, build))
    }
}
