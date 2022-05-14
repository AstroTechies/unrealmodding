pub mod base_export;
pub mod class_export;
pub mod data_table_export;
pub mod enum_export;
pub mod function_export;
pub mod level_export;
pub mod normal_export;
pub mod property_export;
pub mod raw_export;
pub mod string_table_export;
pub mod struct_export;

use std::io::Cursor;

use enum_dispatch::enum_dispatch;

use self::{
    base_export::BaseExport, class_export::ClassExport, data_table_export::DataTableExport,
    enum_export::EnumExport, function_export::FunctionExport, level_export::LevelExport,
    normal_export::NormalExport, property_export::PropertyExport, raw_export::RawExport,
    string_table_export::StringTableExport, struct_export::StructExport,
};
use super::error::Error;
use super::Asset;

#[enum_dispatch]
pub trait ExportNormalTrait {
    fn get_normal_export<'a>(&'a self) -> Option<&'a NormalExport>;
    fn get_normal_export_mut<'a>(&'a mut self) -> Option<&'a mut NormalExport>;
}

#[enum_dispatch]
pub trait ExportBaseTrait {
    fn get_base_export<'a>(&'a self) -> &'a BaseExport;
    fn get_base_export_mut<'a>(&'a mut self) -> &'a mut BaseExport;
}

#[macro_export]
macro_rules! implement_get {
    ($name:ident) => {
        impl ExportNormalTrait for $name {
            fn get_normal_export<'a>(&'a self) -> Option<&'a NormalExport> {
                Some(&self.normal_export)
            }

            fn get_normal_export_mut<'a>(&'a mut self) -> Option<&'a mut NormalExport> {
                Some(&mut self.normal_export)
            }
        }

        impl ExportBaseTrait for $name {
            fn get_base_export<'a>(&'a self) -> &'a BaseExport {
                &self.normal_export.base_export
            }

            fn get_base_export_mut<'a>(&'a mut self) -> &'a mut BaseExport {
                &mut self.normal_export.base_export
            }
        }
    };
}

#[enum_dispatch]
pub trait ExportTrait {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error>;
}

#[enum_dispatch(ExportTrait, ExportNormalTrait, ExportBaseTrait)]
pub enum Export {
    BaseExport,
    ClassExport,
    EnumExport,
    LevelExport,
    NormalExport,
    PropertyExport,
    RawExport,
    StringTableExport,
    StructExport,
    FunctionExport,
    DataTableExport,
}

impl Export {}

impl Clone for Export {
    fn clone(&self) -> Self {
        match self {
            Self::BaseExport(arg0) => Self::BaseExport(arg0.clone()),
            Self::ClassExport(arg0) => Self::ClassExport(arg0.clone()),
            Self::EnumExport(arg0) => Self::EnumExport(arg0.clone()),
            Self::LevelExport(arg0) => Self::LevelExport(arg0.clone()),
            Self::NormalExport(arg0) => Self::NormalExport(arg0.clone()),
            Self::PropertyExport(arg0) => Self::PropertyExport(arg0.clone()),
            Self::RawExport(arg0) => Self::RawExport(arg0.clone()),
            Self::StringTableExport(arg0) => Self::StringTableExport(arg0.clone()),
            Self::StructExport(arg0) => Self::StructExport(arg0.clone()),
            Self::FunctionExport(arg0) => Self::FunctionExport(arg0.clone()),
            Self::DataTableExport(arg0) => Self::DataTableExport(arg0.clone()),
        }
    }
}