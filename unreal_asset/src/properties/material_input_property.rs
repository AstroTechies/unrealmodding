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
use crate::unreal_types::{FName, Guid};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MaterialExpression {
    name: FName,
    extras: Vec<u8>,
    output_index: i32,
    input_name: FName,
    expression_name: FName,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ColorMaterialInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
    pub value: ColorProperty,
}
impl_property_data_trait!(ColorMaterialInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ScalarMaterialInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
    pub value: OrderedFloat<f32>,
}
impl_property_data_trait!(ScalarMaterialInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ShadingModelMaterialInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
    pub value: u32,
}
impl_property_data_trait!(ShadingModelMaterialInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct VectorMaterialInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
    pub value: VectorProperty,
}
impl_property_data_trait!(VectorMaterialInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Vector2MaterialInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
    pub value: Vector2DProperty,
}
impl_property_data_trait!(Vector2MaterialInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ExpressionInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
}
impl_property_data_trait!(ExpressionInputProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MaterialAttributesInputProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub material_expression: MaterialExpression,
}
impl_property_data_trait!(MaterialAttributesInputProperty);

impl MaterialExpression {
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;
        asset.read_i32::<LittleEndian>()?;

        let value = ColorProperty::new(asset, name.clone(), false, 0)?;

        Ok(ColorMaterialInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;
        asset.read_i32::<LittleEndian>()?;

        let value = asset.read_f32::<LittleEndian>()?;

        Ok(ScalarMaterialInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = asset.read_u32::<LittleEndian>()?;
        Ok(ShadingModelMaterialInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = VectorProperty::new(asset, name.clone(), false, 0)?;
        Ok(VectorMaterialInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        asset.read_i32::<LittleEndian>()?;
        let value = Vector2DProperty::new(asset, name.clone(), false, 0)?;
        Ok(Vector2MaterialInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        Ok(ExpressionInputProperty {
            name,
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let material_expression = MaterialExpression::new(asset, name.clone(), false)?;

        Ok(MaterialAttributesInputProperty {
            name,
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
