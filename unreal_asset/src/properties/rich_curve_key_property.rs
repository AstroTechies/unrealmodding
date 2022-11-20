use std::mem::size_of;

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid};

#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveInterpMode {
    Linear,
    Constant,
    Cubic,
    None,
}

#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentMode {
    Auto,
    User,
    Break,
    None,
}

#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentWeightMode {
    WeightedNone,
    WeightedArrive,
    WeightedLeave,
    WeightedBoth,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct RichCurveKeyProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,

    pub interp_mode: RichCurveInterpMode,
    pub tangent_mode: RichCurveTangentMode,
    pub tangent_weight_mode: RichCurveTangentWeightMode,
    pub time: OrderedFloat<f32>,
    pub value: OrderedFloat<f32>,
    pub arrive_tangent: OrderedFloat<f32>,
    pub arrive_tangent_weight: OrderedFloat<f32>,
    pub leave_tangent: OrderedFloat<f32>,
    pub leave_tangent_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(RichCurveKeyProperty);

impl RichCurveKeyProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let interp_mode = RichCurveInterpMode::try_from(asset.read_i8()?)?; // todo: implement normal errors
        let tangent_mode = RichCurveTangentMode::try_from(asset.read_i8()?)?;
        let tangent_weight_mode = RichCurveTangentWeightMode::try_from(asset.read_i8()?)?;

        let time = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let value = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let arrive_tangent = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let arrive_tangent_weight = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let leave_tangent = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let leave_tangent_weight = OrderedFloat(asset.read_f32::<LittleEndian>()?);

        Ok(RichCurveKeyProperty {
            name,
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
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i8(self.interp_mode.into())?;
        asset.write_i8(self.tangent_mode.into())?;
        asset.write_i8(self.tangent_weight_mode.into())?;
        asset.write_f32::<LittleEndian>(self.time.0)?;
        asset.write_f32::<LittleEndian>(self.value.0)?;
        asset.write_f32::<LittleEndian>(self.arrive_tangent.0)?;
        asset.write_f32::<LittleEndian>(self.arrive_tangent_weight.0)?;
        asset.write_f32::<LittleEndian>(self.leave_tangent.0)?;
        asset.write_f32::<LittleEndian>(self.leave_tangent_weight.0)?;
        Ok(size_of::<f32>() * 6 + size_of::<i8>() * 3)
    }
}
