use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct WeightedRandomSamplerProperty {
    name: FName,
    property_guid: Option<Guid>,
    prob: Vec<f32>,
    alias: Vec<i32>,
    total_weight: f32
}

#[derive(Hash, PartialEq, Eq)]
pub struct SkeletalMeshSamplingLODBuiltDataProperty {
    name: FName,
    property_guid: Option<Guid>,
    sampler_property: WeightedRandomSamplerProperty
}

impl WeightedRandomSamplerProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let size = cursor.read_i32::<LittleEndian>()?;
        let prob = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            prob[i] = cursor.read_f32::<LittleEndian>()?;
        }

        let size = cursor.read_i32::<LittleEndian>()?;
        let alias = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            alias[i] = cursor.read_i32::<LittleEndian>()?;
        }

        let total_weight = cursor.read_f32::<LittleEndian>()?;

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
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let sampler_property = WeightedRandomSamplerProperty::new(name, cursor, false, 0)?;

        Ok(SkeletalMeshSamplingLODBuiltDataProperty {
            name,
            property_guid,
            sampler_property
        })
    }
}