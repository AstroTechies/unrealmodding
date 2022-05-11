use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::Export,
    flags::EObjectFlags,
    properties::{
        guid_property::GuidProperty, int_property::BoolProperty, object_property::ObjectProperty,
        str_property::NameProperty, struct_property::StructProperty, Property,
    },
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Asset, Import,
};
use unreal_modintegrator::write_asset;
use unreal_pak::PakFile;
use uuid::Uuid;

use crate::assets::ACTOR_TEMPLATE_ASSET;

use super::{game_to_absolute, get_asset};

#[allow(clippy::ptr_arg)]
pub(crate) fn handle_linked_actor_components(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    linked_actors_maps: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut actor_asset = Asset::new(ACTOR_TEMPLATE_ASSET.to_vec(), None);
    actor_asset.engine_version = VER_UE4_23;
    actor_asset
        .parse_data()
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let object_property_template = actor_asset
        .exports
        .get(6)
        .and_then(|e| cast!(Export, RawExport, e))
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted LevelTemplate"))?;

    let template_export = actor_asset
        .exports
        .get(5)
        .and_then(|e| cast!(Export, NormalExport, e))
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted LevelTemplate"))?;

    let scs_node_template = actor_asset
        .exports
        .get(10)
        .and_then(|e| cast!(Export, NormalExport, e))
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted LevelTemplate"))?;

    let mut new_components = HashMap::new();

    for linked_actor_map in &linked_actors_maps {
        let linked_actors_map = linked_actor_map
            .as_object()
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid linked_actor_components"))?;
        for (name, components) in linked_actors_map.iter() {
            let components = components.as_array().ok_or_else(|| {
                io::Error::new(ErrorKind::Other, "Invalid linked_actor_components")
            })?;

            let entry = new_components.entry(name.clone()).or_insert_with(Vec::new);
            for component in components {
                let component_name = component.as_str().ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Invalid linked_actor_components")
                })?;
                entry.push(String::from(component_name));
            }
        }
    }

    for (name, components) in &new_components {
        let name = game_to_absolute(name)
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid asset name"))?;
        let mut asset = get_asset(integrated_pak, game_paks, &name, VER_UE4_23)?;

        let mut scs_location = None;
        let mut bgc_location = None;
        let mut cdo_location = None;
        let mut node_offset = 0;

        for i in 0..asset.exports.len() {
            let export = &asset.exports[i];
            if let Some(normal_export) = cast!(Export, NormalExport, export) {
                let name = match normal_export.base_export.class_index.is_import() {
                    true => {
                        let import = asset
                            .get_import(normal_export.base_export.class_index)
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?;
                        import.class_name.content.clone()
                    }
                    false => String::new(),
                };

                match name.as_str() {
                    "SimpleConstructionScript" => scs_location = Some(i),
                    "BlueprintGeneratedClass" => bgc_location = Some(i),
                    "SCS_Node" => node_offset += 0,
                    _ => {}
                };
                if (EObjectFlags::RF_CLASS_DEFAULT_OBJECT
                    & EObjectFlags::from_bits(normal_export.base_export.object_flags)
                        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid export"))?)
                    == EObjectFlags::RF_CLASS_DEFAULT_OBJECT
                {
                    cdo_location = Some(i);
                }
            }
        }

        let (scs_location, bgc_location, cdo_location) = {
            (
                scs_location.ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Unable to find SimpleConstructionScript")
                })? as i32,
                bgc_location.ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Unable to find BlueprintGeneratedClass")
                })? as i32,
                cdo_location
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Unable to find CDO"))?
                    as i32,
            )
        };

        let object_property_import = asset
            .find_import_no_index(
                &FName::from_slice("/Script/CoreUObject"),
                &FName::from_slice("Clawss"),
                &FName::from_slice("ObjectProperty"),
            )
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?;
        let _default_object_property_import = asset
            .find_import_no_index(
                &FName::from_slice("/Script/CoreUObject"),
                &FName::from_slice("ObjectProperty"),
                &FName::from_slice("Default__ObjectProperty"),
            )
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?;

        let scs_node_import = asset
            .find_import_no_index(
                &FName::from_slice("/Script/CoreUObject"),
                &FName::from_slice("Class"),
                &FName::from_slice("SCS_Node"),
            )
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?;
        let default_scs_node_import = asset
            .find_import_no_index(
                &FName::from_slice("/Script/CoreUObject"),
                &FName::from_slice("SCS_Node"),
                &FName::from_slice("Default__SCS_Node"),
            )
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?;
        let none_ref = asset
            .search_name_reference(&String::from("None"))
            .ok_or_else(|| {
                io::Error::new(ErrorKind::Other, "Name reference to \"None\" not found")
            })?
            .to_le_bytes();
        asset.add_fname("bAutoActivate");

        for component_path_raw in components {
            let mut object_property_template = object_property_template.clone();
            let mut template_export = template_export.clone();
            let mut scs_node_template = scs_node_template.clone();

            let component_path = component_path_raw.as_str();
            let component = Path::new(component_path_raw)
                .file_stem()
                .and_then(|e| e.to_str())
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Invalid linked actor component")
                })?;

            let (component_path, component) = match component.contains('.') {
                true => {
                    let split: Vec<&str> = component.split('.').collect();
                    (split[0], &split[1][..split[1].len() - 2])
                }
                false => (component_path, component_path),
            };

            asset.add_fname(component_path);
            asset.add_name_reference(String::from("Default__") + component + "_C", false);
            asset.add_name_reference(String::from(component) + "_C", false);
            asset.add_name_reference(String::from(component) + "_GEN_VARIABLE", false);
            asset.add_fname(component);
            asset.add_fname("SCS_Node");

            let package_import = Import {
                class_package: FName::from_slice("/Script/CoreUObject"),
                class_name: FName::from_slice("Package"),
                outer_index: PackageIndex::new(0),
                object_name: FName::from_slice(component_path),
            };
            let package_import = asset.add_import(package_import);

            let blueprint_generated_class_import = Import {
                class_package: FName::from_slice("/Script/Engine"),
                class_name: FName::from_slice("BlueprintGeneratedClass"),
                outer_index: package_import,
                object_name: FName::new(String::from(component) + "_C", 0),
            };
            let blueprint_generated_class_import =
                asset.add_import(blueprint_generated_class_import);

            let default_import = Import {
                class_package: FName::from_slice(component_path),
                class_name: FName::new(String::from(component) + "_C", 0),
                outer_index: package_import,
                object_name: FName::new(String::from("Default__") + component + "_C", 0),
            };
            let default_import = asset.add_import(default_import);

            template_export.base_export.class_index = blueprint_generated_class_import;
            template_export.base_export.object_name =
                FName::new(String::from(component) + "_GEN_VARIABLE", 0);
            template_export.base_export.template_index = default_import;

            object_property_template.base_export.class_index =
                PackageIndex::new(object_property_import);
            object_property_template.base_export.object_name = FName::from_slice("SCS_Node");
            object_property_template.base_export.template_index =
                PackageIndex::new(default_scs_node_import);

            let mut raw_data = Vec::new();

            // Here we specify the raw data for our ObjectProperty category, including necessary flags and such
            // magic numbers?
            raw_data.extend(none_ref);
            raw_data.extend(vec![
                0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00,
                0x00, 0x00,
            ]);
            raw_data.extend(none_ref);
            raw_data.push(0);
            raw_data.extend(blueprint_generated_class_import.index.to_le_bytes());

            object_property_template.base_export.outer_index = PackageIndex::new(bgc_location + 1);
            template_export.base_export.outer_index = PackageIndex::new(bgc_location + 1);
            scs_node_template.base_export.outer_index = PackageIndex::new(scs_location + 1);

            template_export
                .base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(bgc_location + 1));
            template_export
                .base_export
                .serialization_before_create_dependencies
                .push(blueprint_generated_class_import);
            template_export
                .base_export
                .serialization_before_create_dependencies
                .push(default_import);
            template_export
                .base_export
                .create_before_create_dependencies
                .push(PackageIndex::new(bgc_location + 1));
            template_export.extras = [0u8; 4].to_vec();
            template_export.properties = Vec::from([BoolProperty {
                name: FName::from_slice("bAutoActivate"),
                property_guid: None,
                duplication_index: 0,
                value: true,
            }
            .into()]);
            asset.exports.push(template_export.into());

            let exports_len = asset.exports.len() as i32;
            let cdo_export = cast!(
                Export,
                NormalExport,
                &mut asset.exports[cdo_location as usize]
            )
            .expect("Corrupted memory");
            cdo_export
                .base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(exports_len));

            object_property_template
                .base_export
                .create_before_serialization_dependencies
                .push(blueprint_generated_class_import);
            object_property_template
                .base_export
                .create_before_create_dependencies
                .push(PackageIndex::new(bgc_location + 1));
            object_property_template.data = raw_data;
            asset.exports.push(object_property_template.into());

            node_offset += 1;
            scs_node_template.base_export.object_name =
                FName::new(String::from("SCS_Node"), node_offset);
            scs_node_template.extras = [0u8; 4].to_vec();
            scs_node_template
                .base_export
                .create_before_serialization_dependencies
                .push(blueprint_generated_class_import);
            scs_node_template
                .base_export
                .create_before_serialization_dependencies
                .push(PackageIndex::new(asset.exports.len() as i32 - 1));
            scs_node_template
                .base_export
                .serialization_before_create_dependencies
                .push(PackageIndex::new(scs_node_import));
            scs_node_template
                .base_export
                .serialization_before_create_dependencies
                .push(PackageIndex::new(default_scs_node_import));
            scs_node_template
                .base_export
                .create_before_create_dependencies
                .push(PackageIndex::new(scs_location + 1));
            scs_node_template.properties = Vec::from([
                ObjectProperty {
                    name: FName::from_slice("ComponentClass"),
                    property_guid: None,
                    duplication_index: 0,
                    value: blueprint_generated_class_import,
                }
                .into(),
                ObjectProperty {
                    name: FName::from_slice("ComponentTemplate"),
                    property_guid: None,
                    duplication_index: 0,
                    value: PackageIndex::new(asset.exports.len() as i32 - 1),
                }
                .into(),
                StructProperty {
                    name: FName::from_slice("VariableGuid"),
                    struct_type: Some(FName::from_slice("Guid")),
                    struct_guid: None,
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: false,
                    value: Vec::from([GuidProperty {
                        name: FName::from_slice("VariableGuid"),
                        property_guid: None,
                        duplication_index: 0,
                        value: Uuid::new_v4().as_bytes().to_owned(),
                    }
                    .into()]),
                }
                .into(),
                NameProperty {
                    name: FName::from_slice("InternalVariableName"),
                    property_guid: None,
                    duplication_index: 0,
                    value: FName::from_slice(component),
                }
                .into(),
            ]);
            asset.exports.push(scs_node_template.into());
            let scs_node_template_index = asset.exports.len() - 1;

            let exports_len = asset.exports.len() as i32;
            let bgc = cast!(
                Export,
                StructExport,
                &mut asset.exports[bgc_location as usize]
            )
            .expect("Corrupted memory");
            bgc.children.push(PackageIndex::new(exports_len - 1));

            let scs_export = cast!(
                Export,
                NormalExport,
                &mut asset.exports[scs_location as usize]
            )
            .expect("Corrupted memory");

            scs_export
                .base_export
                .create_before_serialization_dependencies
                .push(PackageIndex::new(exports_len));
            scs_export
                .base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(exports_len));

            let mut new_scs_node_name_index = None;
            for property in &mut scs_export.properties {
                if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                    match array_property.name.content.as_str() {
                        "AllNodes" | "RootNodes" => {
                            new_scs_node_name_index = Some(array_property.value.len() as i32 + 1);
                            array_property.value.push(
                                ObjectProperty {
                                    name: array_property.name.clone(),
                                    property_guid: None,
                                    duplication_index: 0,
                                    value: PackageIndex::new(exports_len), // SCS_Node
                                }
                                .into(),
                            )
                        }
                        _ => {}
                    }
                }
            }
            let new_scs_node_name_index = new_scs_node_name_index
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted ActorTemplate"))?;
            cast!(
                Export,
                NormalExport,
                &mut asset.exports[scs_node_template_index]
            )
            .expect("Corrupted memory")
            .base_export
            .object_name
            .index = new_scs_node_name_index;
        }

        write_asset(integrated_pak, &asset, &name)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
