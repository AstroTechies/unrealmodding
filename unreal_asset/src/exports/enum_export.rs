//! Enum export

use std::collections::HashMap;

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_proc_macro::FNameContainer;

use crate::custom_version::FCoreObjectVersion;
use crate::exports::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::implement_get;
use crate::object_version::ObjectVersion;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::fname::FName;
use crate::Error;

/// Enum cpp form
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECppForm {
    /// Regular
    Regular,
    /// Namespaced
    Namespaced,
    /// Enum class
    EnumClass,
}

/// Enum
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UEnum {
    /// Enum names
    pub names: Vec<(FName, i64)>,
    /// Enum cpp form
    #[container_ignore]
    pub cpp_form: ECppForm,
}

impl UEnum {
    /// Read a `UEnum` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut names = Vec::new();

        if asset.get_object_version() < ObjectVersion::VER_UE4_TIGHTLY_PACKED_ENUMS {
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

        let cpp_form = match asset.get_object_version() < ObjectVersion::VER_UE4_ENUM_CLASS_SUPPORT
        {
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

    /// Write a `UEnum` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.names.len() as i32)?;
        if asset.get_object_version() < ObjectVersion::VER_UE4_TIGHTLY_PACKED_ENUMS {
            // todo: a better algorithm?
            let mut names_map = HashMap::with_capacity(self.names.len());
            for (name, index) in &self.names {
                names_map.insert(*index, name.clone());
            }
            for i in 0..names_map.len() {
                if let Some(name) = names_map.get(&(i as i64)) {
                    asset.write_fname(name)?;
                }
            }
        } else if asset.get_custom_version::<FCoreObjectVersion>().version
            < FCoreObjectVersion::EnumProperties as i32
        {
            for (name, index) in &self.names {
                asset.write_fname(name)?;
                asset.write_u8(*index as u8)?;
            }
        } else {
            for (name, index) in &self.names {
                asset.write_fname(name)?;
                asset.write_i64::<LittleEndian>(*index)?;
            }
        }

        if asset.get_object_version() < ObjectVersion::VER_UE4_ENUM_CLASS_SUPPORT {
            asset.write_i32::<LittleEndian>(match self.cpp_form == ECppForm::Namespaced {
                true => 1,
                false => 0,
            })?;
        } else {
            asset.write_u8(self.cpp_form.into())?;
        }
        Ok(())
    }
}

/// Enum export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumExport {
    /// Base normal export
    pub normal_export: NormalExport,
    /// Enum value
    pub value: UEnum,
}

implement_get!(EnumExport);

impl EnumExport {
    /// Read an `EnumExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
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
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;
        asset.write_i32::<LittleEndian>(0)?;
        self.value.write(asset)?;
        Ok(())
    }
}
