use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::error::Error;
use crate::{
    implement_get,
    properties::{struct_property::StructProperty, Property, PropertyDataTrait},
    unreal_types::FName,
};

use super::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
};
use crate::exports::ExportTrait;

#[derive(Clone)]
pub struct DataTable {
    pub data: Vec<StructProperty>,
}

impl DataTable {
    pub fn new(data: Vec<StructProperty>) -> Self {
        DataTable { data }
    }
}

#[derive(Clone)]
pub struct DataTableExport {
    pub normal_export: NormalExport,
    pub table: DataTable,
}

implement_get!(DataTableExport);

impl DataTableExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;

        let mut decided_struct_type = FName::new(String::from("Generic"), 0);
        for data in &normal_export.properties {
            if let Property::ObjectProperty(property) = data {
                if property.name.content.as_str() == "RowStruct" && property.value.is_import() {
                    if let Some(import) = asset.get_import(property.value) {
                        decided_struct_type = import.object_name.clone();
                    }
                }
            }
        }

        asset.read_i32::<LittleEndian>()?;
        let num_entires = asset.read_i32::<LittleEndian>()? as usize;
        let mut data = Vec::with_capacity(num_entires);
        for _i in 0..num_entires {
            let row_name = asset.read_fname()?;
            let next_struct = StructProperty::custom_header(
                asset,
                row_name,
                1,
                0,
                Some(decided_struct_type.clone()),
                None,
                None,
            )?;
            data.push(next_struct);
        }

        let table = DataTable::new(data);

        Ok(DataTableExport {
            normal_export,
            table,
        })
    }
}

impl ExportTrait for DataTableExport {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;

        let mut decided_struct_type = FName::from_slice("Generic");
        for data in &self.normal_export.properties {
            if data.get_name().content.as_str() == "RowStruct" {
                if let Property::ObjectProperty(prop) = data {
                    if let Some(import) = asset.get_import(prop.value) {
                        decided_struct_type = import.object_name.clone();
                        break;
                    }
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
