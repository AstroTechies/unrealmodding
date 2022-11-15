use std::collections::HashMap;
use std::io::SeekFrom;
use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::exports::{
    base_export::BaseExport, struct_export::StructExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::flags::EClassFlags;
use crate::object_version::ObjectVersion;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, PackageIndex};

#[derive(Clone)]
pub struct SerializedInterfaceReference {
    pub class: i32,
    pub pointer_offset: i32,
    pub implemented_by_k2: bool,
}

impl SerializedInterfaceReference {
    pub fn new(class: i32, pointer_offset: i32, implemented_by_k2: bool) -> Self {
        SerializedInterfaceReference {
            class,
            pointer_offset,
            implemented_by_k2,
        }
    }
}

#[derive(Clone)]
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
    pub class_default_object: PackageIndex,
}

impl ClassExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let struct_export = StructExport::from_base(base, asset)?;

        let num_func_index_entries = asset.read_i32::<LittleEndian>()? as usize;
        let mut func_map = HashMap::with_capacity(num_func_index_entries);
        for _i in 0..num_func_index_entries {
            let name = asset.read_fname()?;
            let function_export = PackageIndex::new(asset.read_i32::<LittleEndian>()?);

            func_map.insert(name, function_export);
        }

        let mut class_flags = EClassFlags::from_bits(asset.read_u32::<LittleEndian>()?)
            .ok_or_else(|| Error::invalid_file("Invalid class flags".to_string()))?;
        if asset.get_object_version() < ObjectVersion::VER_UE4_CLASS_NOTPLACEABLE_ADDED {
            class_flags ^= EClassFlags::CLASS_NOT_PLACEABLE
        }

        let class_within = PackageIndex::new(asset.read_i32::<LittleEndian>()?);
        let class_config_name = asset.read_fname()?;

        let mut interfaces_start = None;
        if asset.get_object_version()
            < ObjectVersion::VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING
        {
            interfaces_start = Some(asset.position());
            let num_interfaces = asset.read_i32::<LittleEndian>()?;
            asset.seek(SeekFrom::Start(
                interfaces_start.unwrap()
                    + size_of::<i32>() as u64
                    + num_interfaces as u64 * (size_of::<i32>() as u64 * 3),
            ))?;
        }

        let class_generated_by = PackageIndex::new(asset.read_i32::<LittleEndian>()?);
        let current_offset = asset.position();

        if asset.get_object_version()
            < ObjectVersion::VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING
        {
            asset.seek(SeekFrom::Start(interfaces_start.unwrap()))?;
        }
        let num_interfaces = asset.read_i32::<LittleEndian>()? as usize;
        let mut interfaces = Vec::with_capacity(num_interfaces);
        for _i in 0..num_interfaces {
            interfaces.push(SerializedInterfaceReference::new(
                asset.read_i32::<LittleEndian>()?,
                asset.read_i32::<LittleEndian>()?,
                asset.read_i32::<LittleEndian>()? == 1,
            ));
        }

        if asset.get_object_version()
            < ObjectVersion::VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING
        {
            asset.seek(SeekFrom::Start(current_offset))?;
        }

        let deprecated_force_script_order = asset.read_i32::<LittleEndian>()? == 1;
        asset.read_i64::<LittleEndian>()?; // none

        let cooked = match asset.get_object_version() >= ObjectVersion::VER_UE4_ADD_COOKED_TO_UCLASS
        {
            true => Some(asset.read_i32::<LittleEndian>()? == 1),
            false => None,
        };

        let class_default_object = PackageIndex::new(asset.read_i32::<LittleEndian>()?);

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
            class_default_object,
        })
    }

    fn serialize_interfaces<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.interfaces.len() as i32)?;
        for interface in &self.interfaces {
            asset.write_i32::<LittleEndian>(interface.class)?;
            asset.write_i32::<LittleEndian>(interface.pointer_offset)?;
            asset.write_i32::<LittleEndian>(match interface.implemented_by_k2 {
                true => 1,
                false => 0,
            })?;
        }
        Ok(())
    }
}

impl ExportNormalTrait for ClassExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        Some(&self.struct_export.normal_export)
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        Some(&mut self.struct_export.normal_export)
    }
}

impl ExportBaseTrait for ClassExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        &self.struct_export.normal_export.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        &mut self.struct_export.normal_export.base_export
    }
}

impl ExportTrait for ClassExport {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_export.write(asset)?;

        asset.write_i32::<LittleEndian>(self.func_map.len() as i32)?;
        for (name, index) in &self.func_map {
            asset.write_fname(name)?;
            asset.write_i32::<LittleEndian>(index.index)?;
        }

        let serializing_class_flags =
            match asset.get_object_version() < ObjectVersion::VER_UE4_CLASS_NOTPLACEABLE_ADDED {
                true => self.class_flags ^ EClassFlags::CLASS_NOT_PLACEABLE,
                false => self.class_flags,
            };
        asset.write_u32::<LittleEndian>(serializing_class_flags.bits())?;

        asset.write_i32::<LittleEndian>(self.class_within.index)?;
        asset.write_fname(&self.class_config_name)?;

        if asset.get_object_version()
            < ObjectVersion::VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING
        {
            self.serialize_interfaces(asset)?;
        }
        asset.write_i32::<LittleEndian>(self.class_generated_by.index)?;

        if asset.get_object_version()
            >= ObjectVersion::VER_UE4_UCLASS_SERIALIZE_INTERFACES_AFTER_LINKING
        {
            self.serialize_interfaces(asset)?;
        }

        asset.write_i32::<LittleEndian>(match self.deprecated_force_script_order {
            true => 1,
            false => 0,
        })?;
        asset.write_fname(&FName::from_slice("None"))?;

        if asset.get_object_version() >= ObjectVersion::VER_UE4_ADD_COOKED_TO_UCLASS {
            asset.write_i32::<LittleEndian>(
                match self.cooked.ok_or_else(|| {
                    Error::no_data(
                        "engine_version >= UE4_ADD_COOKED_TO_UCLASS but cooked is None".to_string(),
                    )
                })? {
                    true => 1,
                    false => 0,
                },
            )?;
        }

        asset.write_i32::<LittleEndian>(self.class_default_object.index)?;
        Ok(())
    }
}
