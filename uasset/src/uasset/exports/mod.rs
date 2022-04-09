pub mod unknown_export;
pub mod level_export;
pub mod normal_export;
pub mod string_table_export;
pub mod enum_export;
pub mod struct_export;
pub mod property_export;
pub mod class_export;
pub mod raw_export;
pub mod data_table_export;

use std::io::{Cursor};

use enum_dispatch::enum_dispatch;

use self::{unknown_export::UnknownExport, class_export::ClassExport, enum_export::EnumExport, level_export::LevelExport, normal_export::NormalExport, property_export::PropertyExport, raw_export::RawExport, string_table_export::StringTableExport, struct_export::StructExport, data_table_export::DataTableExport};
use super::error::Error;
use super::Asset;

#[enum_dispatch]
pub trait ExportNormalTrait {
    fn get_normal_export<'a>(&'a self) -> Option<&'a NormalExport>;
    fn get_normal_export_mut<'a>(&'a mut self) -> Option<&'a mut NormalExport>;
}

#[enum_dispatch]
pub trait ExportUnknownTrait {
    fn get_unknown_export<'a>(&'a self) -> &'a UnknownExport;
    fn get_unknown_export_mut<'a>(&'a mut self) -> &'a mut UnknownExport;
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

        impl ExportUnknownTrait for $name {
            fn get_unknown_export<'a>(&'a self) -> &'a UnknownExport {
                &self.normal_export.unknown_export
            }

            fn get_unknown_export_mut<'a>(&'a mut self) -> &'a mut UnknownExport {
                &mut self.normal_export.unknown_export
            }
        }
    };
}

#[enum_dispatch]
trait ExportTrait {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error>;
}

#[enum_dispatch(ExportTrait, ExportNormalTrait, ExportUnknownTrait)]
pub enum Export {
    UnknownExport,
    ClassExport,
    EnumExport,
    LevelExport,
    NormalExport,
    PropertyExport,
    RawExport,
    StringTableExport,
    StructExport,
    DataTableExport
}

impl Export {
}