use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

use super::{color_property::ColorProperty, vector_property::{Vector2DProperty, VectorProperty}};

pub struct MaterialExpression {
    name: FName,
    extras: Vec<u8>,
    output_index: i32,
    input_name: FName,
    expression_name: FName
}

#[derive(Hash, PartialEq, Eq)]
pub struct ColorMaterialInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression,
    value: ColorProperty
}

#[derive(Hash, PartialEq, Eq)]
pub struct ScalarMaterialInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression,
    value: f32
}

#[derive(Hash, PartialEq, Eq)]
pub struct ShadingModelMaterialInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression,
    value: u32
}

#[derive(Hash, PartialEq, Eq)]
pub struct VectorMaterialInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression,
    value: VectorProperty
}

#[derive(Hash, PartialEq, Eq)]
pub struct Vector2MaterialInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression,
    value: Vector2DProperty
}

#[derive(Hash, PartialEq, Eq)]
pub struct ExpressionInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression
}

#[derive(Hash, PartialEq, Eq)]
pub struct MaterialAttributesInputProperty {
    name: FName,
    property_guid: Option<Guid>,
    material_expression: MaterialExpression
}

impl MaterialExpression {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let output_index = cursor.read_i32::<LittleEndian>()?;
        let input_name = asset.read_fname()?;
        let mut extras = [0u8; 20];
        cursor.read_exact(&mut extras)?;
        let expression_name = asset.read_fname()?;

        Ok(MaterialExpression {
            name,
            output_index,
            input_name,
            extras: extras.to_vec(),
            expression_name
        })
    }
}

impl ColorMaterialInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;
        cursor.read_i32::<LittleEndian>()?;

        let value = ColorProperty::new(name, cursor, false)?;

        Ok(ColorMaterialInputProperty {
            name,
            property_guid,
            material_expression,
            value
        })
    }
}

impl ScalarMaterialInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;
        cursor.read_i32::<LittleEndian>()?;

        let value = cursor.read_f32::<LittleEndian>()?;

        Ok(ScalarMaterialInputProperty {
            name,
            property_guid,
            material_expression,
            value
        })
    }
}

impl ShadingModelMaterialInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;

        cursor.read_i32::<LittleEndian>()?;
        let value = cursor.read_u32::<LittleEndian>()?;
        Ok(ShadingModelMaterialInputProperty {
            name,
            property_guid,
            material_expression,
            value
        })
    }
}

impl VectorMaterialInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;

        cursor.read_i32::<LittleEndian>()?;
        let value = VectorProperty::new(name, cursor, false)?;
        Ok(VectorMaterialInputProperty {
            name,
            property_guid,
            material_expression,
            value
        })
    }
}

impl Vector2MaterialInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;

        cursor.read_i32::<LittleEndian>()?;
        let value = Vector2DProperty::new(name, cursor, false)?;
        Ok(Vector2MaterialInputProperty {
            name,
            property_guid,
            material_expression,
            value
        })
    }
}

impl ExpressionInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;

        Ok(ExpressionInputProperty {
            name,
            property_guid,
            material_expression
        })
    }
}

impl MaterialAttributesInputProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let material_expression = MaterialExpression::new(name, cursor, false, asset)?;

        Ok(MaterialAttributesInputProperty {
            name,
            property_guid,
            material_expression
        })
    }
}