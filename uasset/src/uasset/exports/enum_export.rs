use std::io::{Cursor,};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::Error;
use crate::uasset::custom_version::FCoreObjectVersion;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::ue4version::{VER_UE4_ENUM_CLASS_SUPPORT, VER_UE4_TIGHTLY_PACKED_ENUMS};
use crate::uasset::unreal_types::FName;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::uasset::exports::unknown_export::UnknownExport;

use super::ExportNormalTrait;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECppForm {
    Regular,
    Namespaced,
    EnumClass
}

pub struct Enum {
    pub names: Vec<(FName, i64)>,
    pub cpp_form: ECppForm
}

impl Enum {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let mut names = Vec::new();

        if asset.engine_version < VER_UE4_TIGHTLY_PACKED_ENUMS {
            let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
            for i in 0..num_entries {
                let name = asset.read_fname()?;
                names.push((name, i as i64));
            }
        } else {
            let custom_version = asset.get_custom_version::<FCoreObjectVersion>();
            if custom_version.version < FCoreObjectVersion::EnumProperties as i32 {
                let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
                for i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.cursor.read_u8()?;
                    names.push((name, index as i64));
                }
            } else {
                let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
                for i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.cursor.read_i64::<LittleEndian>()?;
                    names.push((name, index));
                }
            }
        }

        let cpp_form = match asset.engine_version < VER_UE4_ENUM_CLASS_SUPPORT {
            true => {
                let is_namespace = asset.cursor.read_i32::<LittleEndian>()? == 1;
                match is_namespace {
                    true => ECppForm::Namespaced,
                    false => ECppForm::Regular
                }
            }
            false => asset.cursor.read_u8()?.try_into()?
        };

        Ok(Enum {
            names,
            cpp_form
        })
    }
}

pub struct EnumExport {
    pub normal_export: NormalExport,

    pub value: Enum
}

implement_get!(EnumExport);

impl EnumExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        let normal_export = NormalExport::from_unk(unk, asset)?;
        asset.cursor.read_i32::<LittleEndian>()?;

        let value = Enum::new(asset)?;
        Ok(EnumExport {
            normal_export,
            value
        })
    }
}