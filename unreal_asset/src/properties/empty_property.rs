//! Empty unversioned property

use crate::{
    error::Error, reader::asset_writer::AssetWriter, types::FName, unversioned::ancestry::Ancestry,
};

use super::{PropertyDataTrait, PropertyTrait};

/// Empty unversioned property
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmptyProperty {
    /// Property type name
    pub type_name: FName,
    /// Property name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
}

impl EmptyProperty {
    /// Create a new `EmptyProperty` instance
    pub fn new(type_name: FName, name: FName, ancestry: Ancestry) -> Self {
        EmptyProperty {
            type_name,
            name,
            ancestry,
        }
    }
}

impl PropertyDataTrait for EmptyProperty {
    fn get_name(&self) -> FName {
        self.name.clone()
    }

    fn get_name_mut(&mut self) -> &mut FName {
        &mut self.name
    }

    fn get_duplication_index(&self) -> i32 {
        0
    }

    fn get_property_guid(&self) -> Option<crate::types::Guid> {
        None
    }

    fn get_ancestry(&self) -> &Ancestry {
        &self.ancestry
    }

    fn get_ancestry_mut(&mut self) -> &mut Ancestry {
        &mut self.ancestry
    }
}

impl PropertyTrait for EmptyProperty {
    fn write<Writer: AssetWriter>(
        &self,
        _asset: &mut Writer,
        _include_header: bool,
    ) -> Result<usize, Error> {
        Ok(0)
    }
}
