use std::collections::HashMap;
use std::io::{Cursor, Error, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::custom_version::FCoreObjectVersion;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::flags::EClassFlags;
use crate::uasset::fproperty::FProperty;
use crate::uasset::kismet::KismetExpression;
use crate::uasset::ue4version::VER_UE4_16;
use crate::uasset::unreal_types::{FName, PackageIndex};
use crate::uasset::uproperty::UField;

use super::ExportNormalTrait;

pub struct StructExport {
    pub normal_export: NormalExport,

    pub field: UField,
    pub super_struct: PackageIndex,
    pub children: Vec<PackageIndex>,
    pub loaded_properties: Vec<FProperty>,
    pub script_bytecode: Option<Vec<KismetExpression>>,
    pub script_bytecode_size: i32,
    pub script_bytecode_raw: Option<Vec<u8>>,
}

implement_get!(StructExport);

impl StructExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;
        asset.cursor.read_i32::<LittleEndian>()?;
        let field = UField::new(asset)?;
        let super_struct = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);

        let num_index_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut children = Vec::with_capacity(num_index_entries as usize);
        for i in 0..num_index_entries as usize {
            children[i] = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        }

        let mut loaded_properties = match asset.get_custom_version("FCoreObjectVersion").map(|e| e.version >= FCoreObjectVersion::FProperties as i32).unwrap_or(false) {
            true => {
                let num_props = asset.cursor.read_i32::<LittleEndian>()?;
                let mut props = Vec::with_capacity(num_props as usize);
                for i in 0..num_props as usize {
                    props[i] = FProperty::new(asset)?;
                }
                props
            },
            false => Vec::new()
        };

        let script_bytecode_size = asset.cursor.read_i32::<LittleEndian>()?; // number of bytes in deserialized memory
        let script_storage_size = asset.cursor.read_i32::<LittleEndian>()?; // number of bytes in total
        let start_offset = asset.cursor.position();

        let mut script_bytecode = None;
        if asset.engine_version >= VER_UE4_16 {
            script_bytecode = StructExport::read_bytecode(asset, start_offset, script_storage_size).ok();
        }

        let script_bytecode_raw = match &script_bytecode {
            Some(_) => None,
            None => {
                asset.cursor.seek(SeekFrom::Start(start_offset));
                let mut data = Vec::with_capacity(script_storage_size as usize);
                asset.cursor.read_exact(&mut data)?;
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
            script_bytecode_raw
        })
    }

    fn read_bytecode(asset: &mut Asset, start_offset: u64, storage_size: i32) -> Result<Vec<KismetExpression>, Error> {
        let mut code = Vec::new();
        while (asset.cursor.position() - start_offset) < storage_size as u64 {
            code.push(KismetExpression::new(asset)?);
        }
        Ok(code)
    }
}