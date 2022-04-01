pub mod unknown_export;
pub mod level_export;

use std::io::Error;

use enum_dispatch::enum_dispatch;

use self::unknown_export::UnknownExport;

#[enum_dispatch]
trait ExportTrait {}

#[enum_dispatch(ExportTrait)]
pub enum Export {
    UnknownExport,
    // LevelExport,
    // StringTableExport,
    // EnumExport,
    // FunctionExport,
    // DataTableExport,
    // ClassExport,
    // PropertyExport,
    // NormalExport
}

impl Export {
    pub fn new(unk: UnknownExport) -> Self {
        Export::UnknownExport(unk)
    }

    // pub fn new(export_class_type: &str) -> Result<Self, Error> {
    //     match export_class_type {
    //         "Level" => {

    //         },
    //         "StringTable" => {

    //         },
    //         "Enum" => {

    //         },
    //         "UserDefinedEnum" => {

    //         },
    //         "Function" => {

    //         },
    //         _ => {

    //         }
    //     }
    // }
}