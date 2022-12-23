#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameNumber {
    pub value: i32,
}

impl FrameNumber {
    pub fn new(value: i32) -> Self {
        FrameNumber { value }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameRate {
    pub numerator: i32,
    pub denominator: i32,
}

impl FrameRate {
    pub fn new(numerator: i32, denominator: i32) -> Self {
        FrameRate {
            numerator,
            denominator,
        }
    }
}

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    error::Error,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
};

/// Enum CoreUObject.ERangeBoundTypes
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum ERangeBoundTypes {
    Exclusive = 0,
    Inclusive = 1,
    Open = 2,
    MAX = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FFrameNumberRangeBound {
    pub ty: ERangeBoundTypes,
    pub value: FrameNumber,
}

impl FFrameNumberRangeBound {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let ty: ERangeBoundTypes = ERangeBoundTypes::try_from(asset.read_i8()?)?;
        let value = FrameNumber::new(asset.read_i32::<LittleEndian>()?);

        Ok(FFrameNumberRangeBound { ty, value })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i8(self.ty as i8)?;
        asset.write_i32::<LittleEndian>(self.value.value)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FFrameNumberRange {
    pub lower_bound: FFrameNumberRangeBound,
    pub upper_bound: FFrameNumberRangeBound,
}

impl FFrameNumberRange {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let lower_bound = FFrameNumberRangeBound::new(asset)?;
        let upper_bound = FFrameNumberRangeBound::new(asset)?;

        Ok(FFrameNumberRange {
            lower_bound,
            upper_bound,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.lower_bound.write(asset)?;
        self.upper_bound.write(asset)?;
        Ok(())
    }
}
