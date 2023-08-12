//! Structs related to movies

use byteorder::LE;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    error::Error,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
};

/// Frame number
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameNumber {
    /// Value
    pub value: i32,
}

impl FrameNumber {
    /// Create a new `FrameNumber` instance
    pub fn new(value: i32) -> Self {
        FrameNumber { value }
    }
}

/// Frame rate
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameRate {
    /// Numerator
    pub numerator: i32,
    /// Denominator
    pub denominator: i32,
}

impl FrameRate {
    /// Create a new `FrameRate` instance
    ///
    /// # Examples
    ///
    /// This creates a `FrameRate` structure describing 60 fps
    /// ```
    /// # use unreal_asset_base as unreal_asset;
    /// use unreal_asset::types::movie::FrameRate;
    /// let frame_rate = FrameRate::new(60, 1);
    /// ```
    pub fn new(numerator: i32, denominator: i32) -> Self {
        FrameRate {
            numerator,
            denominator,
        }
    }
}

/// Enum CoreUObject.ERangeBoundTypes
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum ERangeBoundTypes {
    /// Exclusive range
    Exclusive = 0,
    /// Inclusive range
    Inclusive = 1,
    /// Open range
    Open = 2,
    /// Max value
    MAX = 3,
}

/// Frame number bound by range
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FFrameNumberRangeBound {
    /// Binding range
    pub ty: ERangeBoundTypes,
    /// Frame number
    pub value: FrameNumber,
}

impl FFrameNumberRangeBound {
    /// Read a `FFrameNumberRangeBound` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let ty: ERangeBoundTypes = ERangeBoundTypes::try_from(asset.read_i8()?)?;
        let value = FrameNumber::new(asset.read_i32::<LE>()?);

        Ok(FFrameNumberRangeBound { ty, value })
    }

    /// Write a `FFrameNumberRangeBound` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i8(self.ty as i8)?;
        asset.write_i32::<LE>(self.value.value)?;
        Ok(())
    }
}

/// Frame number range
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FFrameNumberRange {
    /// Lower bound
    pub lower_bound: FFrameNumberRangeBound,
    /// Upper bound
    pub upper_bound: FFrameNumberRangeBound,
}

impl FFrameNumberRange {
    /// Read a `FFrameNumberRange` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let lower_bound = FFrameNumberRangeBound::new(asset)?;
        let upper_bound = FFrameNumberRangeBound::new(asset)?;

        Ok(FFrameNumberRange {
            lower_bound,
            upper_bound,
        })
    }

    /// Write a `FFrameNumberRange` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.lower_bound.write(asset)?;
        self.upper_bound.write(asset)?;
        Ok(())
    }
}
