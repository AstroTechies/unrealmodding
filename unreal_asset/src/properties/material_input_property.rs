//! Material input property

use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{
    color_property::ColorProperty,
    vector_property::{Vector2DProperty, VectorProperty},
    PropertyTrait,
};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Material expression
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MaterialExpression {
    /// Name
    pub name: FName,
    /// Extra data
    pub extras: Vec<u8>,
    /// Output index
    pub output_index: i32,
    /// Input name
    pub input_name: FName,
    /// Material expression name
    pub expression_name: FName,
}

/// Color material input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ColorMaterialInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
    /// Color value
    pub value: ColorProperty,
}
impl_property_data_trait!(ColorMaterialInputProperty);

/// Scalar material input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ScalarMaterialInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
    /// Scalar value
    pub value: OrderedFloat<f32>,
}
impl_property_data_trait!(ScalarMaterialInputProperty);

/// Shading model material input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ShadingModelMaterialInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
    /// Shading model value
    pub value: u32,
}
impl_property_data_trait!(ShadingModelMaterialInputProperty);

/// Vector material input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct VectorMaterialInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
    /// Vector value
    pub value: VectorProperty,
}
impl_property_data_trait!(VectorMaterialInputProperty);

/// Vector2 material input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Vector2MaterialInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
    /// Vector2D value
    pub value: Vector2DProperty,
}
impl_property_data_trait!(Vector2MaterialInputProperty);

/// Expression input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ExpressionInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
}
impl_property_data_trait!(ExpressionInputProperty);

/// Material attributes input property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MaterialAttributesInputProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Material expression
    pub material_expression: MaterialExpression,
}
impl_property_data_trait!(MaterialAttributesInputProperty);

impl MaterialExpression {
    /// Read a `MaterialExpression` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        _include_header: bool,
    ) -> Result<Self, Error> {
        let output_index = asset.read_i32::<LittleEndian>()?;
        let input_name = asset.read_fname()?;
        let mut extras = [0u8; 20];
        asset.read_exact(&mut extras)?;
        let expression_name = asset.read_fname()?;

        Ok(MaterialExpression {
            name,
            output_index,
            input_name,
            extras: extras.to_vec(),
            expression_name,
        })
    }

    /// Write a `MaterialExpression` to an asset
    pub fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        _include_header: bool,
    ) -> Result<usize, Error> {
        asset.write_i32::<LittleEndian>(self.output_index)?;
        asset.write_fname(&self.input_name)?;
        asset.write_all(&self.extras)?;
        asset.write_fname(&self.expression_name)?;
        Ok(size_of::<i32>() * 4 + size_of::<i32>() + 20)
    }
}

impl ColorMaterialInputProperty {
    /// Read a `ColorMaterialInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;
        asset.read_i32::<LittleEndian>()?;

        let value = ColorProperty::new(
            asset,
            name.clone(),
            ancestry.with_parent(name.clone()),
            false,
            0,
        )?;

        Ok(ColorMaterialInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
            value,
        })
    }
}

impl PropertyTrait for ColorMaterialInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let exp_len = self.material_expression.write(asset, false)?;
        asset.write_i32::<LittleEndian>(0)?;
        let value_len = self.value.write(asset, false)?;
        Ok(exp_len + value_len + size_of::<i32>())
    }
}

impl ScalarMaterialInputProperty {
    /// Read a `ScalarMaterialInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;
        asset.read_i32::<LittleEndian>()?;

        let value = asset.read_f32::<LittleEndian>()?;

        Ok(ScalarMaterialInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
            value: OrderedFloat(value),
        })
    }
}

impl PropertyTrait for ScalarMaterialInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let exp_len = self.material_expression.write(asset, false)?;
        asset.write_i32::<LittleEndian>(0)?;
        asset.write_f32::<LittleEndian>(self.value.0)?;
        Ok(exp_len + size_of::<f32>() + size_of::<i32>())
    }
}

impl ShadingModelMaterialInputProperty {
    /// Read a `ShadingModelMaterialInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = asset.read_u32::<LittleEndian>()?;
        Ok(ShadingModelMaterialInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
            value,
        })
    }
}

impl PropertyTrait for ShadingModelMaterialInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let exp_len = self.material_expression.write(asset, false)?;
        asset.write_i32::<LittleEndian>(0)?;
        asset.write_u32::<LittleEndian>(self.value)?;
        Ok(exp_len + size_of::<u32>() + size_of::<i32>())
    }
}

impl VectorMaterialInputProperty {
    /// Read a `VectorMaterialInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = VectorProperty::new(
            asset,
            name.clone(),
            ancestry.with_parent(name.clone()),
            false,
            0,
        )?;
        Ok(VectorMaterialInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
            value,
        })
    }
}

impl PropertyTrait for VectorMaterialInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let exp_len = self.material_expression.write(asset, false)?;
        asset.write_i32::<LittleEndian>(0)?;
        let value_len = self.value.write(asset, false)?;
        Ok(exp_len + value_len + size_of::<i32>())
    }
}

impl Vector2MaterialInputProperty {
    /// Read a `Vector2MaterialInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = Vector2DProperty::new(
            asset,
            name.clone(),
            ancestry.with_parent(name.clone()),
            false,
            0,
        )?;
        Ok(Vector2MaterialInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
            value,
        })
    }
}

impl PropertyTrait for Vector2MaterialInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let exp_len = self.material_expression.write(asset, false)?;
        asset.write_i32::<LittleEndian>(0)?;
        let value_len = self.value.write(asset, false)?;
        Ok(exp_len + value_len + size_of::<i32>())
    }
}

impl ExpressionInputProperty {
    /// Read a `ExpressionInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        Ok(ExpressionInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
        })
    }
}

impl PropertyTrait for ExpressionInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        self.material_expression.write(asset, false)
    }
}

impl MaterialAttributesInputProperty {
    /// Read a `MaterialAttributesInputProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        Ok(MaterialAttributesInputProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            material_expression,
        })
    }
}

impl PropertyTrait for MaterialAttributesInputProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        self.material_expression.write(asset, false)
    }
}
