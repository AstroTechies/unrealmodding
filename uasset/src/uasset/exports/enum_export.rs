use std::collections::HashMap;
use std::io::{Cursor,};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::Error;
use crate::uasset::custom_version::FCoreObjectVersion;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::ue4version::{VER_UE4_ENUM_CLASS_SUPPORT, VER_UE4_TIGHTLY_PACKED_ENUMS};
use crate::uasset::unreal_types::FName;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::uasset::exports::ExportTrait;
use crate::uasset::exports::unknown_export::UnknownExport;

use super::ExportNormalTrait;
use super::ExportUnknownTrait;

#[derive(Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECppForm {
    Regular,
    Namespaced,
    EnumClass
}

pub struct UEnum {
    pub names: Vec<(FName, i64)>,
    pub cpp_form: ECppForm
}

impl UEnum {
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

        Ok(UEnum {
            names,
            cpp_form
        })
    }

    pub fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        cursor.write_i32::<LittleEndian>(self.names.len() as i32)?;
        if asset.engine_version < VER_UE4_TIGHTLY_PACKED_ENUMS {
            // todo: a better algorithm?
            let mut names_map = HashMap::with_capacity(self.names.len());
            for (name, index) in &self.names {
                names_map.insert(*index, name.clone());
            }
            for i in 0..names_map.len() {
                if let Some(name) = names_map.get(&(i as i64)) {
                    asset.write_fname(cursor, name)?;
                }
            }
        } else if asset.get_custom_version::<FCoreObjectVersion>().version < FCoreObjectVersion::EnumProperties as i32 {
            for (name, index) in &self.names {
                asset.write_fname(cursor, name)?;
                cursor.write_u8(*index as u8)?;
            }
        } else {
            for (name, index) in &self.names {
                asset.write_fname(cursor, name)?;
                cursor.write_i64::<LittleEndian>(*index)?;
            }
        }

        if asset.engine_version < VER_UE4_ENUM_CLASS_SUPPORT {
            cursor.write_i32::<LittleEndian>(match self.cpp_form == ECppForm::Namespaced {
                true => 1,
                false => 0
            })?;
        } else {
            cursor.write_u8(self.cpp_form.into())?;
        }
        Ok(())
    }
}

pub struct EnumExport {
    pub normal_export: NormalExport,

    pub value: UEnum
}

implement_get!(EnumExport);

impl EnumExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        let normal_export = NormalExport::from_unk(unk, asset)?;
        asset.cursor.read_i32::<LittleEndian>()?;

        let value = UEnum::new(asset)?;
        Ok(EnumExport {
            normal_export,
            value
        })
    }
}

impl ExportTrait for EnumExport {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(0)?;
        self.value.write(asset, cursor)?;
        Ok(())
    }
}