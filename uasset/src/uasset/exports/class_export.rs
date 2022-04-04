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

pub struct SerializedInterfaceReference {
    class: i32,
    pointer_offset: i32,
    implemented_by_k2: bool
}

impl SerializedInterfaceReference {
    pub fn new(class: i32, pointer_offset: i32, implemented_by_k2: bool) -> Self {
        SerializedInterfaceReference { class, pointer_offset, implemented_by_k2 }
    }
}

pub struct ClassExport {
    struct_export: StructExport,

    func_map: HashMap<FName, PackageIndex>,
    class_flags: EClassFlags,
    class_within: PackageIndex,
    class_config_name: FName,
    interfaces: Vec<SerializedInterfaceReference>,
    class_generated_by: PackageIndex,
    deprecated_force_script_order: bool,
    cooked: Option<bool>,
    class_default_object: PackageIndex
}

impl ClassExport {
    pub fn from_unk(unk: &UnknownExport, cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let struct_export = StructExport::from_unk(unk, cursor, asset)?;

        let num_func_index_entries = cursor.read_i32::<LittleEndian>()? as usize;
        let mut func_map = HashMap::with_capacity(num_func_index_entries);
        for i in 0..num_func_index_entries {
            let name = asset.read_fname()?;
            let function_export = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);

            func_map.insert(name, function_export);
        }

        let class_flags: EClassFlags = cursor.read_u32::<LittleEndian>().map(|e| match asset.engine_version < VER_UE4_CLASS_NOTPLACEABLE_ADDED {
            true => e ^ EClassFlags::CLASS_NotPlaceable as u32,
            false => e
        })?.try_into().map_err(|e| Error::new(ErrorKind::Other, "Invalid class flags"))?;

        let class_within = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);
        let class_config_name = asset.read_fname()?;
        asset.add_name_reference(class_config_name.content.to_owned(), true);

        let mut interfaces_start = None;
        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            interfaces_start = Some(cursor.position());
            let num_interfaces = cursor.read_i32::<LittleEndian>()?;
            cursor.seek(SeekFrom::Start(interfaces_start.unwrap() + size_of::<i32>() as u64 + num_interfaces as u64 * (size_of::<i32>() as u64 * 3)));
        }

        let class_generated_by = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);
        let current_offset = cursor.position();

        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            cursor.seek(SeekFrom::Start(interfaces_start.unwrap()));
        }
        let num_interfaces = cursor.read_i32::<LittleEndian>()? as usize;
        let mut interfaces = Vec::with_capacity(num_interfaces);
        for i in 0..num_interfaces {
            interfaces[i] = SerializedInterfaceReference::new(cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()? == 1);
        }

        if asset.engine_version < VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING {
            cursor.seek(SeekFrom::Start(current_offset));
        }

        let deprecated_force_script_order = cursor.read_i32::<LittleEndian>()? == 1;
        cursor.read_i64()?; // none

        let cooked = match asset.engine_version >= VER_UE4_ADD_COOKED_TO_UCLASS {
            true => Some(cursor.read_i32::<LittleEndian>()? == 1),
            false => None
        };

        let class_default_object = PackageIndex::new(cursor.read_i32::<LittleEndian>()?);

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