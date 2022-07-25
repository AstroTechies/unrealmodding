use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::custom_version::FCoreObjectVersion;
use crate::exports::base_export::BaseExport;
use crate::exports::normal_export::NormalExport;
use crate::exports::ExportTrait;
use crate::implement_get;
use crate::ue4version::{VER_UE4_ENUM_CLASS_SUPPORT, VER_UE4_TIGHTLY_PACKED_ENUMS};
use crate::unreal_types::FName;
use crate::Error;
use byteorder::{LittleEndian, WriteBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::HashMap;
use std::io::Cursor;

use super::ExportBaseTrait;
use super::ExportNormalTrait;

#[derive(Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECppForm {
    Regular,
    Namespaced,
    EnumClass,
}

#[derive(Clone)]
pub struct UEnum {
    pub names: Vec<(FName, i64)>,
    pub cpp_form: ECppForm,
}

impl UEnum {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut names = Vec::new();

        if asset.get_engine_version() < VER_UE4_TIGHTLY_PACKED_ENUMS {
            let num_entries = asset.read_i32::<LittleEndian>()?;
            for i in 0..num_entries {
                let name = asset.read_fname()?;
                names.push((name, i as i64));
            }
        } else {
            let custom_version = asset.get_custom_version::<FCoreObjectVersion>();
            if custom_version.version < FCoreObjectVersion::EnumProperties as i32 {
                let num_entries = asset.read_i32::<LittleEndian>()?;
                for _i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.read_u8()?;
                    names.push((name, index as i64));
                }
            } else {
                let num_entries = asset.read_i32::<LittleEndian>()?;
                for _i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.read_i64::<LittleEndian>()?;
                    names.push((name, index));
                }
            }
        }

        let cpp_form = match asset.get_engine_version() < VER_UE4_ENUM_CLASS_SUPPORT {
            true => {
                let is_namespace = asset.read_i32::<LittleEndian>()? == 1;
                match is_namespace {
                    true => ECppForm::Namespaced,
                    false => ECppForm::Regular,
                }
            }
            false => asset.read_u8()?.try_into()?,
        };

        Ok(UEnum { names, cpp_form })
    }

    pub fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        cursor.write_i32::<LittleEndian>(self.names.len() as i32)?;
        if asset.get_engine_version() < VER_UE4_TIGHTLY_PACKED_ENUMS {
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
        } else if asset.get_custom_version::<FCoreObjectVersion>().version
            < FCoreObjectVersion::EnumProperties as i32
        {
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

        if asset.get_engine_version() < VER_UE4_ENUM_CLASS_SUPPORT {
            cursor.write_i32::<LittleEndian>(match self.cpp_form == ECppForm::Namespaced {
                true => 1,
                false => 0,
            })?;
        } else {
            cursor.write_u8(self.cpp_form.into())?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct EnumExport {
    pub normal_export: NormalExport,

    pub value: UEnum,
}

implement_get!(EnumExport);

impl EnumExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;
        asset.read_i32::<LittleEndian>()?;

        let value = UEnum::new(asset)?;
        Ok(EnumExport {
            normal_export,
            value,
        })
    }
}

impl ExportTrait for EnumExport {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(0)?;
        self.value.write(asset, cursor)?;
        Ok(())
    }
}
