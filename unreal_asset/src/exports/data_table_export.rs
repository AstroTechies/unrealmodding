use byteorder::LittleEndian;

use crate::error::Error;
use crate::exports::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::implement_get;
use crate::properties::{struct_property::StructProperty, Property, PropertyDataTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::FName;

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
        let num_entries = asset.read_i32::<LittleEndian>()? as usize;
        let mut data = Vec::with_capacity(num_entries);

        let parent_name = asset
            .get_parent_class_cached()
            .map(|e| e.parent_class_export_name.clone());

        for _i in 0..num_entries {
            let row_name = asset.read_fname()?;

            let next_struct = StructProperty::custom_header(
                asset,
                row_name,
                parent_name.as_ref(),
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
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;

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
        asset.write_i32::<LittleEndian>(0)?;
        asset.write_i32::<LittleEndian>(self.table.data.len() as i32)?;
        for entry in &self.table.data {
            asset.write_fname(&entry.name)?;
            entry.write_with_type(asset, false, Some(decided_struct_type.clone()))?;
        }

        Ok(())
    }
}
