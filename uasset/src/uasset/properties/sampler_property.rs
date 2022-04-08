use std::io::{Cursor, ErrorKind};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ordered_float::OrderedFloat;

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write, impl_property_data_trait};
use crate::uasset::properties::{PropertyTrait, PropertyDataTrait};

#[derive(Hash, PartialEq, Eq)]
pub struct WeightedRandomSamplerProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub prob: Vec<OrderedFloat<f32>>,
    pub alias: Vec<i32>,
    pub total_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(WeightedRandomSamplerProperty);

#[derive(Hash, PartialEq, Eq)]
pub struct SkeletalMeshAreaWeightedTriangleSampler {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub prob: Vec<OrderedFloat<f32>>,
    pub alias: Vec<i32>,
    pub total_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(SkeletalMeshAreaWeightedTriangleSampler);

#[derive(Hash, PartialEq, Eq)]
pub struct SkeletalMeshSamplingLODBuiltDataProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub sampler_property: WeightedRandomSamplerProperty,
}
impl_property_data_trait!(SkeletalMeshSamplingLODBuiltDataProperty);

impl WeightedRandomSamplerProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, duplication_index: i32) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut prob = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            prob.push(OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?));
        }

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut alias = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            alias.push(asset.cursor.read_i32::<LittleEndian>()?);
        }

        let total_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

        Ok(WeightedRandomSamplerProperty {
            name,
            property_guid,
            duplication_index,
            prob,
            alias,
            total_weight,
        })
    }
}

impl PropertyTrait for WeightedRandomSamplerProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.prob.len() as i32)?;
        for entry in &self.prob {
            cursor.write_f32::<LittleEndian>(entry.0)?;
        }

        cursor.write_i32::<LittleEndian>(self.alias.len() as i32)?;
        for entry in &self.alias {
            cursor.write_i32::<LittleEndian>(*entry)?;
        }

        cursor.write_f32::<LittleEndian>(self.total_weight.0)?;
        Ok(size_of::<i32>() + size_of::<f32>() * self.prob.len() + size_of::<i32>() + size_of::<i32>() * self.alias.len() + size_of::<f32>())
    }
}

impl SkeletalMeshAreaWeightedTriangleSampler {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, duplication_index: i32) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut prob = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            prob.push(OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?));
        }

        let size = asset.cursor.read_i32::<LittleEndian>()?;
        let mut alias = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            alias.push(asset.cursor.read_i32::<LittleEndian>()?);
        }

        let total_weight = OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?);

        Ok(SkeletalMeshAreaWeightedTriangleSampler {
            name,
            property_guid,
            duplication_index,
            prob,
            alias,
            total_weight,
        })
    }
}

impl PropertyTrait for SkeletalMeshAreaWeightedTriangleSampler {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.prob.len() as i32)?;
        for entry in &self.prob {
            cursor.write_f32::<LittleEndian>(entry.0)?;
        }

        cursor.write_i32::<LittleEndian>(self.alias.len() as i32)?;
        for entry in &self.alias {
            cursor.write_i32::<LittleEndian>(*entry)?;
        }

        cursor.write_f32::<LittleEndian>(self.total_weight.0)?;
        Ok(size_of::<i32>() + size_of::<f32>() * self.prob.len() + size_of::<i32>() + size_of::<i32>() * self.alias.len() + size_of::<f32>())
    }
}

impl SkeletalMeshSamplingLODBuiltDataProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, duplication_index: i32) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let sampler_property = WeightedRandomSamplerProperty::new(asset, name.clone(), false, 0, 0)?;

        Ok(SkeletalMeshSamplingLODBuiltDataProperty {
            name,
            property_guid,
            duplication_index,
            sampler_property,
        })
    }
}

impl PropertyTrait for SkeletalMeshSamplingLODBuiltDataProperty {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        self.sampler_property.write(asset, cursor, false)
    }
}