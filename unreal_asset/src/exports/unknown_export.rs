use crate::error::Error;
use crate::exports::ExportTrait;
use crate::unreal_types::{FName, Guid};
use crate::Asset;
use std::io::Cursor;

use super::ExportNormalTrait;
use super::ExportUnknownTrait;

#[derive(Debug, Default, Clone)]
pub struct UnknownExport {
    pub class_index: i32,
    pub super_index: i32,
    pub template_index: i32,
    pub outer_index: i32,
    pub object_name: FName,
    pub object_flags: u32,
    pub serial_size: i64,
    pub serial_offset: i64,
    pub forced_export: bool,
    pub not_for_client: bool,
    pub not_for_server: bool,
    pub package_guid: Guid,
    pub package_flags: u32,
    pub not_always_loaded_for_editor_game: bool,
    pub is_asset: bool,
    pub first_export_dependency_offset: i32,
    pub serialization_before_serialization_dependencies: Vec<i32>,
    pub(crate) serialization_before_serialization_dependencies_size: i32,

    pub create_before_serialization_dependencies: Vec<i32>,
    pub(crate) create_before_serialization_dependencies_size: i32,

    pub serialization_before_create_dependencies: Vec<i32>,
    pub(crate) serialization_before_create_dependencies_size: i32,

    pub create_before_create_dependencies: Vec<i32>,
    pub(crate) create_before_create_dependencies_size: i32
}

impl ExportNormalTrait for UnknownExport {
    fn get_normal_export<'a>(&'a self) -> Option<&'a super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportUnknownTrait for UnknownExport {
    fn get_unknown_export<'a>(&'a self) -> &'a UnknownExport {
        &self
    }

    fn get_unknown_export_mut<'a>(&'a mut self) -> &'a mut UnknownExport {
        self
    }
}

impl ExportTrait for UnknownExport {
    fn write(&self, _asset: &Asset, _cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        Ok(())
    }
}
