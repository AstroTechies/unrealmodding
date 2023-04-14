//! Cloth lod property

use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::{
    error::Error,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::FName,
};

use super::{
    struct_property::StructProperty, vector_property::Vector4Property, PropertyDataTrait,
    PropertyTrait,
};

/// Mesh to mesh vertex data
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MeshToMeshVertData {
    /// Position barycentric coords and distance
    pub position_bary_coords_and_dist: Vector4Property,
    /// Normal barycentric coords and distance
    pub normal_bary_coords_and_dist: Vector4Property,
    /// Tangent barycentric coords and distance
    pub tangent_bary_coords_and_dist: Vector4Property,
    /// Source mesh vertex indices
    pub source_mesh_vert_indices: Vec<u16>,
    /// Weight
    pub weight: OrderedFloat<f32>,
    /// Dummy for alignment
    pub padding: u32,
}

impl MeshToMeshVertData {
    /// Read `MeshToMeshVertData` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let position_bary_coords_and_dist = Vector4Property::new(
            asset,
            FName::from_slice("PositionBaryCoordsAndDist"),
            false,
            0,
        )?;

        let normal_bary_coords_and_dist = Vector4Property::new(
            asset,
            FName::from_slice("NormalBaryCoordsAndDist"),
            false,
            0,
        )?;

        let tangent_bary_coords_and_dist = Vector4Property::new(
            asset,
            FName::from_slice("TangentBaryCoordsAndDist"),
            false,
            0,
        )?;

        let mut source_mesh_vert_indices = Vec::with_capacity(4);
        for _ in 0..4 {
            source_mesh_vert_indices.push(asset.read_u16::<LittleEndian>()?);
        }

        let weight = asset.read_f32::<LittleEndian>()?;
        let padding = asset.read_u32::<LittleEndian>()?;

        Ok(MeshToMeshVertData {
            position_bary_coords_and_dist,
            normal_bary_coords_and_dist,
            tangent_bary_coords_and_dist,
            source_mesh_vert_indices,
            weight: OrderedFloat(weight),
            padding,
        })
    }

    /// Write `MeshToMeshVertData` to an asset
    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut size = 0;
        size += self.position_bary_coords_and_dist.write(asset, false)?;
        size += self.normal_bary_coords_and_dist.write(asset, false)?;
        size += self.tangent_bary_coords_and_dist.write(asset, false)?;

        for i in 0..4 {
            match i < self.source_mesh_vert_indices.len() {
                true => asset.write_u16::<LittleEndian>(self.source_mesh_vert_indices[i]),
                false => asset.write_u16::<LittleEndian>(0),
            }?;
            size += size_of::<u16>();

            asset.write_f32::<LittleEndian>(self.weight.0)?;
            size += size_of::<f32>();
            asset.write_u32::<LittleEndian>(self.padding)?;
            size += size_of::<u32>();
        }

        Ok(size)
    }
}

/// Cloth lod data property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ClothLodDataProperty {
    /// Base struct property
    pub struct_property: StructProperty,
    /// Next lod skin data
    pub transition_up_skin_data: Vec<MeshToMeshVertData>,
    /// Previous lod skin data
    pub transition_down_skin_data: Vec<MeshToMeshVertData>,
}

impl ClothLodDataProperty {
    /// Read a `ClothLodDataProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        parent_name: Option<&FName>,
        _include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let struct_property = StructProperty::custom_header(
            asset,
            name,
            parent_name,
            1,
            duplication_index,
            Some(FName::from_slice("Generic")),
            None,
            None,
        )?;

        let transition_up_skin_data_len = asset.read_i32::<LittleEndian>()?;
        let mut transition_up_skin_data = Vec::with_capacity(transition_up_skin_data_len as usize);
        for _ in 0..transition_up_skin_data_len {
            transition_up_skin_data.push(MeshToMeshVertData::new(asset)?);
        }

        let transition_down_skin_data_len = asset.read_i32::<LittleEndian>()?;
        let mut transition_down_skin_data =
            Vec::with_capacity(transition_down_skin_data_len as usize);
        for _ in 0..transition_down_skin_data_len {
            transition_down_skin_data.push(MeshToMeshVertData::new(asset)?);
        }

        Ok(ClothLodDataProperty {
            struct_property,
            transition_up_skin_data,
            transition_down_skin_data,
        })
    }
}

impl PropertyDataTrait for ClothLodDataProperty {
    fn get_name(&self) -> FName {
        self.struct_property.get_name()
    }

    fn get_name_mut(&mut self) -> &mut FName {
        self.struct_property.get_name_mut()
    }

    fn get_duplication_index(&self) -> i32 {
        self.struct_property.get_duplication_index()
    }

    fn get_property_guid(&self) -> Option<crate::types::Guid> {
        self.struct_property.get_property_guid()
    }
}

impl PropertyTrait for ClothLodDataProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        let mut size = self.struct_property.write_with_type(
            asset,
            include_header,
            Some(FName::from_slice("Generic")),
        )?;

        asset.write_i32::<LittleEndian>(self.transition_up_skin_data.len() as i32)?;
        size += size_of::<i32>();
        for transition_up_skin_data in &self.transition_up_skin_data {
            size += transition_up_skin_data.write(asset)?;
        }

        asset.write_i32::<LittleEndian>(self.transition_down_skin_data.len() as i32)?;
        size += size_of::<i32>();
        for transition_down_skin_data in &self.transition_down_skin_data {
            size += transition_down_skin_data.write(asset)?;
        }

        Ok(size)
    }
}
