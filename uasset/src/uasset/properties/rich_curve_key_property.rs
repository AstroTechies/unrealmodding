use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq)]
#[repr(i8)]
pub enum RichCurveInterpMode {
    Linear,
    Constant,
    Cubic,
    None
}

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq)]
#[repr(i8)]
pub enum RichCurveTangentMode {
    Auto,
    User,
    Break,
    None
}

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq)]
#[repr(i8)]
pub enum RichCurveTangentWeightMode {
    WeightedNone,
    WeightedArrive,
    WeightedLeave,
    WeightedBoth
}

#[derive(Hash, PartialEq, Eq)]
pub struct RichCurveKeyProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,

    pub interp_mode: RichCurveInterpMode,
    pub tangent_mode: RichCurveTangentMode,
    pub tangent_weight_mode: RichCurveTangentWeightMode,
    pub time: OrderedFloat<f32>,
    pub value: OrderedFloat<f32>,
    pub arrive_tangent: OrderedFloat<f32>,
    pub arrive_tangent_weight: OrderedFloat<f32>,
    pub leave_tangent: OrderedFloat<f32>,
    pub leave_tangent_weight: OrderedFloat<f32>
}

impl RichCurveKeyProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let interp_mode = RichCurveInterpMode::try_from(asset.cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?; // todo: implement normal errors
        let tangent_mode = RichCurveTangentMode::try_from(asset.cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        let tangent_weight_mode = RichCurveTangentWeightMode::try_from(asset.cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

        let time = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let value = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let arrive_tangent = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let arrive_tangent_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let leave_tangent = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        let leave_tangent_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

        Ok(RichCurveKeyProperty {
            name,
            property_guid,
            interp_mode,
            tangent_mode,
            tangent_weight_mode,
            time,
            value,
            arrive_tangent,
            arrive_tangent_weight,
            leave_tangent,
            leave_tangent_weight
        })
    }
}
