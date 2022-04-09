use std::io::Cursor;

use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

use crate::{uasset::{properties::{struct_property::StructProperty, object_property::ObjectProperty, Property}, Asset, unreal_types::FName, is_import}, implement_get};
use crate::uasset::error::Error;
use std::io::{ErrorKind};
use crate::uasset::exports::ExportTrait;
use crate::uasset::properties::PropertyTrait;
use crate::uasset::unreal_types::ToFName;
use super::{normal_export::NormalExport, unknown_export::UnknownExport, ExportNormalTrait, ExportUnknownTrait};

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

implement_get!(DataTableExport);

impl DataTableExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;

        let mut decided_struct_type = FName::new(String::from("Generic"), 0);
        for data in &normal_export.properties {
            if let Property::ObjectProperty(property) = data {
                if property.name.content.as_str() == "RowStruct" && is_import(property.value) {
                    if let Some(import) = asset.get_import(property.value) {
                        decided_struct_type = import.object_name.clone();
                    }
                }
            }
        }

        asset.cursor.read_i32::<LittleEndian>()?;
        let num_entires = asset.cursor.read_i32::<LittleEndian>()? as usize;
        let mut data = Vec::with_capacity(num_entires);
        for i in 0..num_entires {
            let row_name = asset.read_fname()?;
            let next_struct = StructProperty::custom_header(asset, row_name, 1, 0,Some(decided_struct_type.clone()), None, None)?;
            data.push(next_struct);
        }

        let table = DataTable::new(data);

        Ok(DataTableExport {
            normal_export,
            table
        })
    }
}

impl ExportTrait for DataTableExport {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;

        let mut decided_struct_type = FName::from_slice("Generic");
        for data in &self.normal_export.properties {
            if data.to_fname().content.as_str() == "RowStruct" {
                match data {
                    Property::ObjectProperty(prop) => if let Some(import) = asset.get_import(prop.value) {
                        decided_struct_type = import.object_name.clone();
                        break;
                    },
                    _ => {}
                }
            }
        }
        cursor.write_i32::<LittleEndian>(0)?;
        cursor.write_i32::<LittleEndian>(self.table.data.len() as i32)?;
        for entry in &self.table.data {
            asset.write_fname(cursor, &entry.name)?;
            entry.write_with_type(asset, cursor, false, Some(decided_struct_type.clone()))?;
        }

        Ok(())
    }
}