use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid};

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct WeightedRandomSamplerProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub prob: Vec<OrderedFloat<f32>>,
    pub alias: Vec<i32>,
    pub total_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(WeightedRandomSamplerProperty);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct SkeletalMeshAreaWeightedTriangleSampler {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub prob: Vec<OrderedFloat<f32>>,
    pub alias: Vec<i32>,
    pub total_weight: OrderedFloat<f32>,
}
impl_property_data_trait!(SkeletalMeshAreaWeightedTriangleSampler);

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct SkeletalMeshSamplingLODBuiltDataProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub sampler_property: WeightedRandomSamplerProperty,
}
impl_property_data_trait!(SkeletalMeshSamplingLODBuiltDataProperty);

impl WeightedRandomSamplerProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.read_i32::<LittleEndian>()?;
        let mut prob = Vec::with_capacity(size as usize);
        for _i in 0..size as usize {
            prob.push(OrderedFloat(asset.read_f32::<LittleEndian>()?));
        }

        let size = asset.read_i32::<LittleEndian>()?;
        let mut alias = Vec::with_capacity(size as usize);
        for _i in 0..size as usize {
            alias.push(asset.read_i32::<LittleEndian>()?);
        }

        let total_weight = OrderedFloat(asset.read_f32::<LittleEndian>()?);

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
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.prob.len() as i32)?;
        for entry in &self.prob {
            asset.write_f32::<LittleEndian>(entry.0)?;
        }

        asset.write_i32::<LittleEndian>(self.alias.len() as i32)?;
        for entry in &self.alias {
            asset.write_i32::<LittleEndian>(*entry)?;
        }

        asset.write_f32::<LittleEndian>(self.total_weight.0)?;
        Ok(size_of::<i32>()
            + size_of::<f32>() * self.prob.len()
            + size_of::<i32>()
            + size_of::<i32>() * self.alias.len()
            + size_of::<f32>())
    }
}

impl SkeletalMeshAreaWeightedTriangleSampler {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.read_i32::<LittleEndian>()?;
        let mut prob = Vec::with_capacity(size as usize);
        for _i in 0..size as usize {
            prob.push(OrderedFloat(asset.read_f32::<LittleEndian>()?));
        }

        let size = asset.read_i32::<LittleEndian>()?;
        let mut alias = Vec::with_capacity(size as usize);
        for _i in 0..size as usize {
            alias.push(asset.read_i32::<LittleEndian>()?);
        }

        let total_weight = OrderedFloat(asset.read_f32::<LittleEndian>()?);

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
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.prob.len() as i32)?;
        for entry in &self.prob {
            asset.write_f32::<LittleEndian>(entry.0)?;
        }

        asset.write_i32::<LittleEndian>(self.alias.len() as i32)?;
        for entry in &self.alias {
            asset.write_i32::<LittleEndian>(*entry)?;
        }

        asset.write_f32::<LittleEndian>(self.total_weight.0)?;
        Ok(size_of::<i32>()
            + size_of::<f32>() * self.prob.len()
            + size_of::<i32>()
            + size_of::<i32>() * self.alias.len()
            + size_of::<f32>())
    }
}

impl SkeletalMeshSamplingLODBuiltDataProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let sampler_property =
            WeightedRandomSamplerProperty::new(asset, name.clone(), false, 0, 0)?;

        Ok(SkeletalMeshSamplingLODBuiltDataProperty {
            name,
            property_guid,
            duplication_index,
            sampler_property,
        })
    }
}

impl PropertyTrait for SkeletalMeshSamplingLODBuiltDataProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        self.sampler_property.write(asset, false)
    }
}
