use std::io::Cursor;

use byteorder::{ReadBytesExt, LittleEndian};

use crate::uasset::{properties::{struct_property::StructProperty, object_property::ObjectProperty, Property}, Asset, unreal_types::FName, is_import};
use std::io::{Error, ErrorKind};
use super::{normal_export::NormalExport, unknown_export::UnknownExport};

pub struct DataTable {
    data: Vec<StructProperty>
}

impl DataTable {
    pub fn new(data: Vec<StructProperty>) -> Self {
        DataTable { data }
    }
}

pub struct DataTableExport {
    normal_export: NormalExport,
    table: DataTable
}

impl DataTableExport {
    pub fn from_unk(unk: &UnknownExport, cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, cursor, asset)?;

        let mut decided_struct_type = FName::new(String::from("Generic"), 0);
        for data in &normal_export.properties {
            if let Property::ObjectProperty(property) = data {
                if property.name.content.as_str() == "RowStruct" && is_import(property.value) {
                    if let Some(import) = asset.get_import(property.value) {
                        decided_struct_type = import.object_name;
                    }
                }
            }
        }

        cursor.read_i32::<LittleEndian>()?;
        let num_entires = cursor.read_i32::<LittleEndian>()? as usize;
        let mut data = Vec::with_capacity(num_entires);
        for i in 0..num_entires {
            let row_name = asset.read_fname()?;
            let next_struct = StructProperty::custom_header(row_name, cursor, 1, asset, Some(decided_struct_type), None, Nones)?;
            data.push(next_struct);
        }

        let table = DataTable::new(data);

        Ok(DataTableExport {
            normal_export,
            table
        })
    }
}