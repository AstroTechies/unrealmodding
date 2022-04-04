use std::io::{Cursor, Error, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::uasset::Asset;
use crate::uasset::custom_version::FCoreObjectVersion;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::fproperty::FProperty;
use crate::uasset::kismet::KismetExpression;
use crate::uasset::ue4version::VER_UE4_16;
use crate::uasset::unreal_types::PackageIndex;
use crate::uasset::uproperty::UField;

pub struct StructExport {
    normal_export: NormalExport,

    field: UField,
    super_struct: PackageIndex,
    children: Vec<PackageIndex>,
    loaded_properties: Vec<FProperty>,
    script_bytecode: Option<Vec<KismetExpression>>,
    script_bytecode_size: i32,
    script_bytecode_raw: Option<Vec<u8>>,
}

impl StructExport {
    pub fn from_unk(unk: &UnknownExport, cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, cursor, asset)?;
        cursor.read_i32::<LittleEndian>()?;
        let field = UField::new(cursor, asset)?;
        let super_struct = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);

        let num_index_entries = cursor.read_i32::<LittleEndian>()?;
        let mut children = Vec::with_capacity(num_index_entries as usize);
        for i in 0..num_index_entries as usize {
            children[i] = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);
        }

        let mut loaded_properties = match asset.get_custom_version("FCoreObjectVersion").map(|e| e.version >= FCoreObjectVersion::FProperties as i32).unwrap_or(false) {
            true => {
                let num_props = cursor.read_i32::<LittleEndian>()?;
                let mut props = Vec::with_capacity(num_props as usize);
                for i in 0..num_props as usize {
                    props[i] = FProperty::new(cursor, asset)?;
                }
                props
            },
            false => Vec::new()
        };

        let script_bytecode_size = cursor.read_i32::<LittleEndian>()?; // number of bytes in deserialized memory
        let script_storage_size = cursor.read_i32::<LittleEndian>()?; // number of bytes in total
        let start_offset = cursor.position();

        let mut script_bytecode = None;
        if asset.engine_version >= VER_UE4_16 {
            script_bytecode = StructExport::read_bytecode(cursor, asset, start_offset, script_storage_size).ok();
        }

        let script_bytecode_raw = match &script_bytecode {
            Some(_) => None,
            None => {
                cursor.seek(SeekFrom::Start(start_offset));
                let mut data = Vec::with_capacity(script_storage_size as usize);
                cursor.read_exact(&mut data)?;
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

    fn read_bytecode(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset, start_offset: u64, storage_size: i32) -> Result<Vec<KismetExpression>, Error> {
        let mut code = Vec::new();
        while (cursor.position() - start_offset) < storage_size as u64 {
            code.push(KismetExpression::new(cursor, asset)?);
        }
        Ok(code)
    }
}