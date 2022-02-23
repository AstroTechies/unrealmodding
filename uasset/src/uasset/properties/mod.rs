pub mod int_property;
pub mod str_property;
pub mod object_property;
pub mod guid_property;
pub mod date_property;
pub mod color_property;
pub mod vector_property;
pub mod struct_property;
pub mod array_property;

use std::{io::{Error, Cursor}, collections::HashMap};
use byteorder::{ReadBytesExt, LittleEndian};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;

use super::Asset;

#[macro_export]
macro_rules! optional_guid {
    ($cursor:ident, $include_header:ident) => {
        match $include_header {
            true => Some($cursor.read_property_guid()?),
            false => None
        };
    };
}

lazy_static! {
    static ref CUSTOM_SERIALIZATION: Vec<String> = HashMap::from([
        String::from("SkeletalMeshSamplingLODBuiltData"),
        String::from("SkeletalMeshAreaWeightedTriangleSampler"),
        String::from("SmartName"),
        String::from("SoftObjectPath"),
        String::from("WeightedRandomSampler"),
        String::from("SoftClassPath"),
        String::from("Color"),
        String::from("ExpressionInput"),
        String::from("MaterialAttributesInput"),
        String::from("ColorMaterialInput"),
        String::from("ScalarMaterialInput"),
        String::from("ShadingModelMaterialInput"),
        String::from("VectorMaterialInput"),
        String::from("Vector2MaterialInput"),
        String::from("GameplayTagContainer"),
        String::from("PerPlatformBool"),
        String::from("PerPlatformInt"),
        String::from("RichCurveKey"),
        String::from("SoftAssetPath"),
        String::from("Timespan"),
        String::from("DateTime"),
        String::from("Guid"),
        String::from("IntPoint"),
        String::from("LinearColor"),
        String::from("Quat"),
        String::from("Rotator"),
        String::from("Vector2D"),
        String::from("Box"),
        String::from("PerPlatformFloat"),
        String::from("Vector4"),
        String::from("Vector"),
        String::from("ViewTargetBlendParams"),
    ]);
}

#[enum_dispatch]
trait PropertyTrait {
}

#[enum_dispatch(PropertyTrait)]
pub enum Property {

}

impl Property {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: Asset, include_header: bool) -> Result<Option<Self>, Error> {
        let offset = cursor.position();
        let name = asset.read_fname()?; // probably should pass cursor instancce there
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?; // probably should pass cursor instance there
        let length = cursor.read_i32::<LittleEndian>()?;
        let duplication_index = cursor.read_i32::<LittleEndian>()?;

        Ok(Some(Property{}))
    }

    pub fn has_custom_serialization(name: String) -> bool {
        CUSTOM_SERIALIZATION.contains(&name)
    }
}