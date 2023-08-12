//! Data table export

use byteorder::LE;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::implement_get;
use crate::properties::{struct_property::StructProperty, Property, PropertyDataTrait};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::fname::FName;
use crate::unversioned::ancestry::Ancestry;
use crate::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};

/// Data table
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTable {
    /// Data
    pub data: Vec<StructProperty>,
}

impl DataTable {
    /// Create a new `DataTable` instance
    pub fn new(data: Vec<StructProperty>) -> Self {
        DataTable { data }
    }
}

/// Data table export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTableExport {
    /// Base normal export
    pub normal_export: NormalExport,
    /// Data table
    pub table: DataTable,
}

implement_get!(DataTableExport);

impl DataTableExport {
    /// Read a `DataTableExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;

        let mut decided_struct_type = FName::from_slice("Generic");
        for data in &normal_export.properties {
            if let Property::ObjectProperty(property) = data {
                if property.name == "RowStruct" && property.value.is_import() {
                    if let Some(import) = asset.get_import(property.value) {
                        decided_struct_type = import.object_name.clone();
                    }
                }
            }
        }

        asset.read_i32::<LE>()?;
        let num_entries = asset.read_i32::<LE>()? as usize;
        let mut data = Vec::with_capacity(num_entries);

        let ancestry = Ancestry::new(base.get_class_type_for_ancestry(asset));

        for _i in 0..num_entries {
            let row_name = asset.read_fname()?;

            let next_struct = StructProperty::custom_header(
                asset,
                row_name,
                ancestry.clone(),
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
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;

        let mut decided_struct_type = FName::from_slice("Generic");
        for data in &self.normal_export.properties {
            if data.get_name() == "RowStruct" {
                if let Property::ObjectProperty(prop) = data {
                    if let Some(import) = asset.get_import(prop.value) {
                        decided_struct_type = import.object_name;
                        break;
                    }
                }
            }
        }
        asset.write_i32::<LE>(0)?;
        asset.write_i32::<LE>(self.table.data.len() as i32)?;
        for entry in &self.table.data {
            asset.write_fname(&entry.name)?;
            entry.write_with_type(asset, false, Some(decided_struct_type.clone()))?;
        }

        Ok(())
    }
}
