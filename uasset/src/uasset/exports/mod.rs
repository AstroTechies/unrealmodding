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

use std::io::{Error, Cursor};

use enum_dispatch::enum_dispatch;

use self::{unknown_export::UnknownExport, class_export::ClassExport, enum_export::EnumExport, level_export::LevelExport, normal_export::NormalExport, property_export::PropertyExport, raw_export::RawExport, string_table_export::StringTableExport, struct_export::StructExport};

use super::Asset;


#[enum_dispatch]
trait ExportTrait {}

#[enum_dispatch(ExportTrait)]
pub enum Export {
    UnknownExport,
    ClassExport,
    EnumExport,
    LevelExport,
    NormalExport,
    PropertyExport,
    RawExport,
    StringTableExport,
    StructExport
}

impl Export {
}