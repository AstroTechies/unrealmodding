use std::io::{Cursor, Error};
use crate::uasset::Asset;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::properties::Property;
use crate::uasset::unreal_types::{FName, Guid};

use super::ExportNormalTrait;

pub struct NormalExport {
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
    pub first_export_dependency: i32,
    pub serialization_before_serialization_dependencies : i32,
    pub create_before_serialization_dependencies: i32,
    pub serialization_before_create_dependencies: i32,
    pub create_before_create_dependencies: i32,
    pub extras: Vec<u8>,

    pub properties: Vec<Property>
}

impl ExportNormalTrait for NormalExport {
    fn get_normal_export< 'a>(&'a self) -> Option<& 'a NormalExport> {
        Some(&self)
    }


    fn get_normal_export_mut< 'a>(&'a mut self) -> Option<& 'a mut NormalExport> {
        Some(self)
    }

}

impl NormalExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        let mut properties = Vec::new();

        while let Some(e) = Property::new(asset, true)? {
            properties.push(e);
        }

        Ok(NormalExport {
            class_index: unk.class_index,
            super_index: unk.super_index,
            template_index: unk.template_index,
            outer_index: unk.outer_index,
            object_name: unk.object_name.clone(),
            object_flags: unk.object_flags,
            serial_size: unk.serial_size,
            serial_offset: unk.serial_offset,
            forced_export: unk.forced_export,
            not_for_client: unk.not_for_client,
            not_for_server: unk.not_for_server,
            package_guid: unk.package_guid,
            package_flags: unk.package_flags,
            not_always_loaded_for_editor_game: unk.not_always_loaded_for_editor_game,
            is_asset: unk.is_asset,
            first_export_dependency: unk.first_export_dependency,
            serialization_before_serialization_dependencies : unk.serialization_before_serialization_dependencies ,
            create_before_serialization_dependencies: unk.create_before_serialization_dependencies,
            serialization_before_create_dependencies: unk.serialization_before_create_dependencies,
            create_before_create_dependencies: unk.create_before_create_dependencies,
            extras: Vec::new(),

            properties
        })
    }
}