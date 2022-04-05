use std::collections::HashMap;
use std::mem::size_of;
use std::io::{Cursor, Error, ErrorKind, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::uasset::Asset;
use crate::uasset::exports::struct_export::StructExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::flags::EClassFlags;
use crate::uasset::ue4version::{VER_UE4_ADD_COOKED_TO_UCLASS, VER_UE4_CLASS_NOTPLACEABLE_ADDED, VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING};
use crate::uasset::unreal_types::{FName, PackageIndex};

use super::ExportNormalTrait;

pub struct SerializedInterfaceReference {
    pub class: i32,
    pub pointer_offset: i32,
    pub implemented_by_k2: bool
}

impl SerializedInterfaceReference {
    pub fn new(class: i32, pointer_offset: i32, implemented_by_k2: bool) -> Self {
        SerializedInterfaceReference { class, pointer_offset, implemented_by_k2 }
    }
}

pub struct ClassExport {
    pub struct_export: StructExport,

    pub func_map: HashMap<FName, PackageIndex>,
    pub class_flags: EClassFlags,
    pub class_within: PackageIndex,
    pub class_config_name: FName,
    pub interfaces: Vec<SerializedInterfaceReference>,
    pub class_generated_by: PackageIndex,
    pub deprecated_force_script_order: bool,
    pub cooked: Option<bool>,
    pub class_default_object: PackageIndex
}

impl ClassExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let struct_export = StructExport::from_unk(unk, asset)?;

        let num_func_index_entries = asset.cursor.read_i32::<LittleEndian>()? as usize;
        let mut func_map = HashMap::with_capacity(num_func_index_entries);
        for i in 0..num_func_index_entries {
            let name = asset.read_fname()?;
            let function_export = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);

            func_map.insert(name, function_export);
        }

        let mut class_flags = EClassFlags::from_bits(asset.cursor.read_u32::<LittleEndian>()?).ok_or(Error::new(ErrorKind::Other, "Invalid class flags"))?;
        if asset.engine_version < VER_UE4_CLASS_NOTPLACEABLE_ADDED {
            class_flags = class_flags ^ EClassFlags::CLASS_NotPlaceable
        }

        let class_within = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let class_config_name = asset.read_fname()?;
        asset.add_name_reference(class_config_name.content.to_owned(), true);

        let mut interfaces_start = None;
        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            interfaces_start = Some(asset.cursor.position());
            let num_interfaces = asset.cursor.read_i32::<LittleEndian>()?;
            asset.cursor.seek(SeekFrom::Start(interfaces_start.unwrap() + size_of::<i32>() as u64 + num_interfaces as u64 * (size_of::<i32>() as u64 * 3)));
        }

        let class_generated_by = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let current_offset = asset.cursor.position();

        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            asset.cursor.seek(SeekFrom::Start(interfaces_start.unwrap()));
        }
        let num_interfaces = asset.cursor.read_i32::<LittleEndian>()? as usize;
        let mut interfaces = Vec::with_capacity(num_interfaces);
        for i in 0..num_interfaces {
            interfaces.push(SerializedInterfaceReference::new(asset.cursor.read_i32::<LittleEndian>()?, asset.cursor.read_i32::<LittleEndian>()?, asset.cursor.read_i32::<LittleEndian>()? == 1));
        }

        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            asset.cursor.seek(SeekFrom::Start(current_offset));
        }

        let deprecated_force_script_order = asset.cursor.read_i32::<LittleEndian>()? == 1;
        asset.cursor.read_i64::<LittleEndian>()?; // none

        let cooked = match asset.engine_version >= VER_UE4_ADD_COOKED_TO_UCLASS {
            true => Some(asset.cursor.read_i32::<LittleEndian>()? == 1),
            false => None
        };

        let class_default_object = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);

        Ok(ClassExport {
            struct_export,

            func_map,
            class_flags,
            class_within,
            class_config_name,
            interfaces,
            class_generated_by,
            deprecated_force_script_order,
            cooked,
            class_default_object
        })
    }
}

impl ExportNormalTrait for ClassExport {
    fn get_normal_export< 'a>(&'a self) -> Option<& 'a super::normal_export::NormalExport> {
        Some(&self.struct_export.normal_export)
    }


    fn get_normal_export_mut< 'a>(&'a mut self) -> Option<& 'a mut super::normal_export::NormalExport> {
        Some(&mut self.struct_export.normal_export)
    }
}