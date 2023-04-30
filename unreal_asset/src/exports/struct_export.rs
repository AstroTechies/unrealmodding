//! Struct export

use std::io::SeekFrom;

use byteorder::LittleEndian;

use crate::custom_version::FCoreObjectVersion;
use crate::engine_version::EngineVersion;
use crate::error::Error;
use crate::exports::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::fproperty::FProperty;
use crate::implement_get;
use crate::kismet::KismetExpression;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::PackageIndex;
use crate::uproperty::UField;

/// Struct export
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructExport {
    /// Base normal export
    pub normal_export: NormalExport,
    /// Field
    pub field: UField,
    /// Super struct
    pub super_struct: PackageIndex,
    /// Children
    pub children: Vec<PackageIndex>,
    /// Loaded properties
    pub loaded_properties: Vec<FProperty>,
    /// Script bytecode, exists if bytecode deserialized successfully
    pub script_bytecode: Option<Vec<KismetExpression>>,
    /// Script bytecode size
    pub script_bytecode_size: i32,
    /// Script bytecode raw, exists if bytecode couldn't deserialize successfully
    pub script_bytecode_raw: Option<Vec<u8>>,
}

implement_get!(StructExport);

impl StructExport {
    /// Read a `StructExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;
        asset.read_i32::<LittleEndian>()?;
        let field = UField::new(asset)?;
        let super_struct = PackageIndex::new(asset.read_i32::<LittleEndian>()?);

        let num_index_entries = asset.read_i32::<LittleEndian>()?;
        let mut children = Vec::with_capacity(num_index_entries as usize);
        for _i in 0..num_index_entries as usize {
            children.push(PackageIndex::new(asset.read_i32::<LittleEndian>()?));
        }

        let loaded_properties = match asset.get_custom_version::<FCoreObjectVersion>().version
            >= FCoreObjectVersion::FProperties as i32
        {
            true => {
                let num_props = asset.read_i32::<LittleEndian>()?;
                let mut props = Vec::with_capacity(num_props as usize);
                for _i in 0..num_props as usize {
                    props.push(FProperty::new(asset)?);
                }
                props
            }
            false => Vec::new(),
        };

        let script_bytecode_size = asset.read_i32::<LittleEndian>()?; // number of bytes in deserialized memory
        let script_storage_size = asset.read_i32::<LittleEndian>()?; // number of bytes in total
        let start_offset = asset.position();

        let mut script_bytecode = None;
        if asset.get_engine_version() >= EngineVersion::VER_UE4_16 {
            script_bytecode =
                StructExport::read_bytecode(asset, start_offset, script_storage_size).ok();
        }

        let script_bytecode_raw = match &script_bytecode {
            Some(_) => None,
            None => {
                asset.seek(SeekFrom::Start(start_offset))?;
                let mut data = vec![0u8; script_storage_size as usize];
                asset.read_exact(&mut data)?;
                Some(data)
            }
        };

        Ok(StructExport {
            normal_export,
            field,

            super_struct,
            children,
            loaded_properties,
            script_bytecode,
            script_bytecode_size,
            script_bytecode_raw,
        })
    }

    /// Read kismet bytecode
    fn read_bytecode<Reader: ArchiveReader>(
        asset: &mut Reader,
        start_offset: u64,
        storage_size: i32,
    ) -> Result<Vec<KismetExpression>, Error> {
        let mut code = Vec::new();
        while (asset.position() - start_offset) < storage_size as u64 {
            code.push(KismetExpression::new(asset)?);
        }
        Ok(code)
    }
}

impl ExportTrait for StructExport {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;
        asset.write_i32::<LittleEndian>(0)?;
        self.field.write(asset)?;

        asset.write_i32::<LittleEndian>(self.super_struct.index)?;
        asset.write_i32::<LittleEndian>(self.children.len() as i32)?;
        for child in &self.children {
            asset.write_i32::<LittleEndian>(child.index)?;
        }

        if asset.get_custom_version::<FCoreObjectVersion>().version
            >= FCoreObjectVersion::FProperties as i32
        {
            asset.write_i32::<LittleEndian>(self.loaded_properties.len() as i32)?;
            for loaded_property in &self.loaded_properties {
                FProperty::write(loaded_property, asset)?;
            }
        }

        if let Some(bytecode) = &self.script_bytecode {
            let len_offset_1 = asset.position();
            asset.write_i32::<LittleEndian>(0)?; // total iCode offset; will be filled after serialization
            let len_offset_2 = asset.position();
            asset.write_i32::<LittleEndian>(0)?; // size on disk; will be filled after serialization

            let mut total_offset = 0;
            let begin = asset.position();
            for expression in bytecode {
                total_offset += KismetExpression::write(expression, asset)?;
            }
            let end = asset.position();

            let total_len = end - begin;
            asset.seek(SeekFrom::Start(len_offset_1))?;
            asset.write_i32::<LittleEndian>(total_offset as i32)?;
            asset.seek(SeekFrom::Start(len_offset_2))?;
            asset.write_i32::<LittleEndian>(total_len as i32)?;
            asset.seek(SeekFrom::Start(end))?;
        } else {
            asset.write_i32::<LittleEndian>(self.script_bytecode_size)?;
            let raw_bytecode = self.script_bytecode_raw.as_ref().ok_or_else(|| {
                Error::no_data("script_bytecode and raw_bytecode are None".to_string())
            })?;
            asset.write_i32::<LittleEndian>(raw_bytecode.len() as i32)?;
            asset.write_all(raw_bytecode)?;
        }

        Ok(())
    }
}
