//! Rich curve key property

use std::mem::size_of;

use byteorder::LE;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Rich curve extrapolation
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum RichCurveExtrapolation {
    /// Cycle
    Cycle = 0,
    /// Cycle with offset
    CycleWithOffset = 1,
    /// Oscillate
    Oscillate = 2,
    /// Linear
    Linear = 3,
    /// Constant
    Constant = 4,
    /// None
    None = 5,
    /// Max
    MAX = 6,
}

/// Rich curve interpolation mode
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveInterpMode {
    /// Linear
    Linear,
    /// Constant
    Constant,
    /// Cubic
    Cubic,
    /// None
    None,
}

/// Rich curve tangent mode
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentMode {
    /// Auto
    Auto,
    /// User
    User,
    /// Break
    Break,
    /// None
    None,
}

/// Rich curve tangent weight mode
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentWeightMode {
    /// None
    WeightedNone,
    /// Arrive
    WeightedArrive,
    /// Leave
    WeightedLeave,
    /// Both
    WeightedBoth,
}

/// Rich curve key property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct RichCurveKeyProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Interpolation mode
    #[container_ignore]
    pub interp_mode: RichCurveInterpMode,
    /// Tangent mode
    #[container_ignore]
    pub tangent_mode: RichCurveTangentMode,
    /// Tangent weight mode
    #[container_ignore]
    pub tangent_weight_mode: RichCurveTangentWeightMode,
    /// Time
    pub time: OrderedFloat<f32>,
    /// Curve key value
    pub value: OrderedFloat<f32>,
    /// Arrive tangent
    pub arrive_tangent: OrderedFloat<f32>,
    /// Arrive tangent weight
    pub arrive_tangent_weight: OrderedFloat<f32>,
    /// Leave tangent
    pub leave_tangent: OrderedFloat<f32>,
    /// Leave tangent weight
    pub leave_tangent_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(RichCurveKeyProperty);

impl RichCurveKeyProperty {
    /// Read a `RichCurveKeyProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let interp_mode = RichCurveInterpMode::try_from(asset.read_i8()?)?; // todo: implement normal errors
        let tangent_mode = RichCurveTangentMode::try_from(asset.read_i8()?)?;
        let tangent_weight_mode = RichCurveTangentWeightMode::try_from(asset.read_i8()?)?;

        let time = OrderedFloat(asset.read_f32::<LE>()?);
        let value = OrderedFloat(asset.read_f32::<LE>()?);
        let arrive_tangent = OrderedFloat(asset.read_f32::<LE>()?);
        let arrive_tangent_weight = OrderedFloat(asset.read_f32::<LE>()?);
        let leave_tangent = OrderedFloat(asset.read_f32::<LE>()?);
        let leave_tangent_weight = OrderedFloat(asset.read_f32::<LE>()?);

        Ok(RichCurveKeyProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            interp_mode,
            tangent_mode,
            tangent_weight_mode,
            time,
            value,
            arrive_tangent,
            arrive_tangent_weight,
            leave_tangent,
            leave_tangent_weight,
        })
    }
}

impl PropertyTrait for RichCurveKeyProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i8(self.interp_mode.into())?;
        asset.write_i8(self.tangent_mode.into())?;
        asset.write_i8(self.tangent_weight_mode.into())?;
        asset.write_f32::<LE>(self.time.0)?;
        asset.write_f32::<LE>(self.value.0)?;
        asset.write_f32::<LE>(self.arrive_tangent.0)?;
        asset.write_f32::<LE>(self.arrive_tangent_weight.0)?;
        asset.write_f32::<LE>(self.leave_tangent.0)?;
        asset.write_f32::<LE>(self.leave_tangent_weight.0)?;
        Ok(size_of::<f32>() * 6 + size_of::<i8>() * 3)
    }
}
