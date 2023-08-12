//! Niagara variable property

use crate::property_prelude::*;

/// Niagara variable property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct NiagaraVariableProperty {
    /// Base struct property
    pub struct_property: StructProperty,
    /// Variable name
    pub variable_name: FName,
    /// Variable offset
    pub variable_offset: i32,
}

impl NiagaraVariableProperty {
    /// Read a `NiagaraVariableProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        _include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let variable_name = asset.read_fname()?;

        let mut properties = Vec::new();
        let mut unversioned_header = UnversionedHeader::new(asset)?;
        let new_ancestry = ancestry.with_parent(name.clone());
        while let Some(property) = Property::new(
            asset,
            new_ancestry.clone(),
            unversioned_header.as_mut(),
            true,
        )? {
            properties.push(property);
        }

        let variable_offset = asset.read_i32::<LE>()?;

        Ok(NiagaraVariableProperty {
            struct_property: StructProperty {
                name,
                ancestry,
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

    fn get_property_guid(&self) -> Option<Guid> {
        self.struct_property.get_property_guid()
    }

    fn get_ancestry(&self) -> &Ancestry {
        self.struct_property.get_ancestry()
    }

    fn get_ancestry_mut(&mut self) -> &mut Ancestry {
        self.struct_property.get_ancestry_mut()
    }
}

impl PropertyTrait for NiagaraVariableProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        _include_header: bool,
    ) -> Result<usize, Error> {
        let begin = asset.position();

        asset.write_fname(&self.variable_name)?;

        let (unversioned_header, sorted_properties) = match generate_unversioned_header(
            asset,
            &self.struct_property.value,
            &self.struct_property.name,
        )? {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        };

        if let Some(unversioned_header) = unversioned_header {
            unversioned_header.write(asset)?;
        }

        let properties = sorted_properties
            .as_ref()
            .unwrap_or(&self.struct_property.value);
        for property in properties.iter() {
            Property::write(property, asset, true)?;
        }

        if !asset.has_unversioned_properties() {
            asset.write_fname(&asset.get_name_map().get_mut().add_fname("None"))?;
        }
        asset.write_i32::<LE>(self.variable_offset)?;

        Ok((asset.position() - begin) as usize)
    }
}

/// Niagara variable with offset property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct NiagaraVariableWithOffsetProperty {
    /// Variable
    pub niagara_variable: NiagaraVariableProperty,
}

impl NiagaraVariableWithOffsetProperty {
    /// Read a `NiagaraVariableWithOffsetProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        Ok(NiagaraVariableWithOffsetProperty {
            niagara_variable: NiagaraVariableProperty::new(
                asset,
                name,
                ancestry,
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

    fn get_property_guid(&self) -> Option<Guid> {
        self.niagara_variable.get_property_guid()
    }

    fn get_ancestry(&self) -> &Ancestry {
        self.niagara_variable.get_ancestry()
    }

    fn get_ancestry_mut(&mut self) -> &mut Ancestry {
        self.niagara_variable.get_ancestry_mut()
    }
}

impl PropertyTrait for NiagaraVariableWithOffsetProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.niagara_variable.write(asset, include_header)
    }
}
