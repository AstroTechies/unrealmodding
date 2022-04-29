use std::io::Cursor;
use std::mem::size_of;

use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    {
        unreal_types::{FName, Guid},
        Asset,
    },
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveInterpMode {
    Linear,
    Constant,
    Cubic,
    None,
}

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentMode {
    Auto,
    User,
    Break,
    None,
}

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum RichCurveTangentWeightMode {
    WeightedNone,
    WeightedArrive,
    WeightedLeave,
    WeightedBoth,
}

#[derive(Hash, Clone, PartialEq, Eq)]
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
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let interp_mode = RichCurveInterpMode::try_from(asset.cursor.read_i8()?)?; // todo: implement normal errors
        let tangent_mode = RichCurveTangentMode::try_from(asset.cursor.read_i8()?)?;
        let tangent_weight_mode = RichCurveTangentWeightMode::try_from(asset.cursor.read_i8()?)?;

        let time = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let value = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let arrive_tangent = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let arrive_tangent_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let leave_tangent = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let leave_tangent_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

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
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i8(self.interp_mode.into())?;
        cursor.write_i8(self.tangent_mode.into())?;
        cursor.write_i8(self.tangent_weight_mode.into())?;
        cursor.write_f32::<LittleEndian>(self.time.0)?;
        cursor.write_f32::<LittleEndian>(self.value.0)?;
        cursor.write_f32::<LittleEndian>(self.arrive_tangent.0)?;
        cursor.write_f32::<LittleEndian>(self.arrive_tangent_weight.0)?;
        cursor.write_f32::<LittleEndian>(self.leave_tangent.0)?;
        cursor.write_f32::<LittleEndian>(self.leave_tangent_weight.0)?;
        Ok(size_of::<f32>() * 6 + size_of::<i8>() * 3)
    }
}
