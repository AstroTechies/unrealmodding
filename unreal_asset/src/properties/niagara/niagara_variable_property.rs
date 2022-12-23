use byteorder::LittleEndian;

use crate::{
    error::Error,
    properties::{struct_property::StructProperty, Property, PropertyDataTrait, PropertyTrait},
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::FName,
};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct NiagaraVariableProperty {
    pub struct_property: StructProperty,
    pub variable_name: FName,
    pub variable_offset: i32,
}

impl NiagaraVariableProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        _include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let variable_name = asset.read_fname()?;

        let mut properties = Vec::new();
        while let Some(property) = Property::new(asset, Some(&name), true)? {
            properties.push(property);
        }

        let variable_offset = asset.read_i32::<LittleEndian>()?;

        Ok(NiagaraVariableProperty {
            struct_property: StructProperty {
                name,
                struct_type: None,
                struct_guid: None,
                property_guid: None,
                duplication_index,
                serialize_none: false,
                value: properties,
            },
            variable_name,
            variable_offset,
        })
    }
}

impl PropertyDataTrait for NiagaraVariableProperty {
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

impl PropertyTrait for NiagaraVariableProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        _include_header: bool,
    ) -> Result<usize, Error> {
        let begin = asset.position();

        asset.write_fname(&self.variable_name)?;

        for property in &self.struct_property.value {
            Property::write(property, asset, true)?;
        }

        asset.write_fname(&FName::from_slice("None"))?;
        asset.write_i32::<LittleEndian>(self.variable_offset)?;

        Ok((asset.position() - begin) as usize)
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct NiagaraVariableWithOffsetProperty {
    pub niagara_variable: NiagaraVariableProperty,
}

impl NiagaraVariableWithOffsetProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        Ok(NiagaraVariableWithOffsetProperty {
            niagara_variable: NiagaraVariableProperty::new(
                asset,
                name,
                include_header,
                length,
                duplication_index,
            )?,
        })
    }
}

impl PropertyDataTrait for NiagaraVariableWithOffsetProperty {
    fn get_name(&self) -> FName {
        self.niagara_variable.get_name()
    }

    fn get_name_mut(&mut self) -> &mut FName {
        self.niagara_variable.get_name_mut()
    }

    fn get_duplication_index(&self) -> i32 {
        self.niagara_variable.get_duplication_index()
    }

    fn get_property_guid(&self) -> Option<crate::types::Guid> {
        self.niagara_variable.get_property_guid()
    }
}

impl PropertyTrait for NiagaraVariableWithOffsetProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.niagara_variable.write(asset, include_header)
    }
}
