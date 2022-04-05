use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct WeightedRandomSamplerProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub prob: Vec<OrderedFloat<f32>>,
    pub alias: Vec<i32>,
    pub total_weight: OrderedFloat<f32>
}

#[derive(Hash, PartialEq, Eq)]
pub struct SkeletalMeshSamplingLODBuiltDataProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub sampler_property: WeightedRandomSamplerProperty
}

impl WeightedRandomSamplerProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut prob = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            prob[i] = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);
        }

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut alias = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            alias[i] = asset.cursor.read_i32::<LittleEndian>()?;
        }

        let total_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

        Ok(WeightedRandomSamplerProperty {
            name,
            property_guid,
            prob,
            alias,
            total_weight
        })
    }
}

impl SkeletalMeshSamplingLODBuiltDataProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let sampler_property = WeightedRandomSamplerProperty::new(asset, name.clone(), false, 0)?;

        Ok(SkeletalMeshSamplingLODBuiltDataProperty {
            name,
            property_guid,
            sampler_property
        })
    }
}
