use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
pub enum RichCurveInterpMode {
    Linear,
    Constant,
    Cubic,
    None
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
pub enum RichCurveTangentMode {
    Auto,
    User,
    Break,
    None
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
pub enum RichCurveTangentWeightMode {
    WeightedNone,
    WeightedArrive,
    WeightedLeave,
    WeightedBoth
}

#[derive(Hash, PartialEq, Eq)]
pub struct RichCurveKeyProperty {
    name: FName,
    property_guid: Option<Guid>,

    interp_mode: RichCurveInterpMode,
    tangent_mode: RichCurveTangentMode,
    tangent_weight_mode: RichCurveTangentWeightMode,
    time: f32,
    value: f32,
    arrive_tangent: f32,
    arrive_tangent_weight: f32,
    leave_tangent: f32,
    leave_tangent_weight: f32
}

impl RichCurveKeyProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let interp_mode = RichCurveInterpMode::try_from(cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?; // todo: implement normal errors
        let tangent_mode = RichCurveTangentMode::try_from(cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        let tangent_weight_mode = RichCurveTangentWeightMode::try_from(cursor.read_i8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

        let time = cursor.read_f32::<LittleEndian>()?;
        let value = cursor.read_f32::<LittleEndian>()?;
        let arrive_tangent = cursor.read_f32::<LittleEndian>()?;
        let arrive_tangent_weight = cursor.read_f32::<LittleEndian>()?;
        let leave_tangent = cursor.read_f32::<LittleEndian>()?;
        let leave_tangent_weight = cursor.read_f32::<LittleEndian>()?;

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