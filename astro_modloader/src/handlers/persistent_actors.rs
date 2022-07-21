use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::{normal_export::NormalExport, Export, ExportBaseTrait, ExportNormalTrait},
    properties::{
        array_property::ArrayProperty, enum_property::EnumProperty, int_property::BoolProperty,
        object_property::ObjectProperty, Property, PropertyDataTrait,
    },
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Asset, Import,
};
use unreal_modintegrator::{
    find_asset,
    helpers::{game_to_absolute, get_asset},
    read_asset, write_asset, IntegratorConfig,
};
use unreal_pak::PakFile;

use crate::astro_integrator::AstroIntegratorConfig;

use super::MAP_PATHS;

#[derive(Default)]
struct ScsNode {
    internal_variable_name: String,
    type_link: PackageIndex,
    attach_parent: Option<PackageIndex>,
    original_category: PackageIndex,
}

#[allow(clippy::ptr_arg)]
pub(crate) fn handle_persistent_actors(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    mod_paks: &mut Vec<PakFile>,
    persistent_actor_arrays: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut level_asset = Asset::new(crate::assets::LEVEL_TEMPLATE_ASSET.to_vec(), None);
    level_asset.engine_version = VER_UE4_23;
    level_asset
        .parse_data()
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let actor_template = cast!(Export, NormalExport, level_asset.exports[2].clone())
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted actor_template"))?;

    let scene_export = cast!(Export, NormalExport, level_asset.exports[11].clone())
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted scene_component"))?;

    let mut persistent_actors = Vec::new();
    for persistent_actors_array in &persistent_actor_arrays {
        let persistent_actors_array = persistent_actors_array
            .as_array()
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?;

        for persistent_actor in persistent_actors_array {
            persistent_actors.push(
                persistent_actor
                    .as_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?,
            );
        }
    }

    for map_path in MAP_PATHS {
        let mut asset = get_asset(integrated_pak, game_paks, &map_path.to_string(), VER_UE4_23)?;

        let mut level_export_index = None;
        for i in 0..asset.exports.len() {
            if cast!(Export, LevelExport, &asset.exports[i]).is_some() {
                level_export_index = Some(i);
                break;
            }
        }
        let level_export_index = level_export_index
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No level export"))?;

        asset.add_fname("bHidden");
        asset.add_fname("bNetAddressable");
        asset.add_fname("CreationMethod");
        asset.add_fname("EComponentCreationMethod");
        asset.add_fname("EComponentCreationMethod::SimpleConstructionScript");
        asset.add_fname("BlueprintCreatedComponents");
        asset.add_fname("AttachParent");
        asset.add_fname("RootComponent");

        for component_path_raw in &persistent_actors {
            let component = Path::new(component_path_raw)
                .file_stem()
                .and_then(|e| e.to_str())
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?;
            let (component_path_raw, component) = match component.contains('.') {
                true => {
                    let split: Vec<&str> = component.split('.').collect();
                    (split[0].to_string(), &split[1][..split[1].len() - 2])
                }
                false => (component_path_raw.to_string(), component),
            };
            let mut actor_template = actor_template.clone();

            asset.add_name_reference(component_path_raw.clone(), false);
            asset.add_name_reference(String::from(component) + "_C", false);
            asset.add_name_reference(String::from("Default__") + component + "_C", false);
            asset.add_fname(component);

            let package_import = Import {
                class_package: asset.add_fname("/Script/CoreUObject"),
                class_name: asset.add_fname("Package"),
                outer_index: PackageIndex::new(0),
                object_name: asset.add_fname(&component_path_raw),
            };
            let package_import = asset.add_import(package_import);

            let blueprint_generated_class_import = Import {
                class_package: asset.add_fname("/Script/Engine"),
                class_name: asset.add_fname("BlueprintGeneratedClass"),
                outer_index: package_import,
                object_name: asset.add_fname(&(String::from(component) + "_C")),
            };
            let blueprint_generated_class_import =
                asset.add_import(blueprint_generated_class_import);

            let default_import = Import {
                class_package: asset.add_fname(&component_path_raw),
                class_name: asset.add_fname(&(String::from(component) + "_C")),
                outer_index: package_import,
                object_name: asset.add_fname(&(String::from("Default__") + component + "_C")),
            };
            let default_import = asset.add_import(default_import);

            actor_template.base_export.class_index = blueprint_generated_class_import;
            actor_template.base_export.object_name = FName::from_slice(component);
            actor_template.base_export.template_index = default_import;
            actor_template.base_export.outer_index =
                PackageIndex::new(level_export_index as i32 + 1); // package index starts from 1

            let actor_asset_path =
                game_to_absolute(AstroIntegratorConfig::GAME_NAME, &component_path_raw)
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid actor path"))?;
            let pak_index = find_asset(mod_paks, &actor_asset_path)
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such asset"))?;
            let actor_asset = read_asset(&mut mod_paks[pak_index], VER_UE4_23, &actor_asset_path)
                .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

            let mut scs_location = None;
            for i in 0..actor_asset.exports.len() {
                let export = &actor_asset.exports[i];
                if let Some(normal_export) = export.get_normal_export() {
                    if normal_export.base_export.class_index.is_import() {
                        let import = asset
                            .get_import(normal_export.base_export.class_index)
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Import not found"))?;
                        if import.object_name.content == "SimpleConstructionScript" {
                            scs_location = Some(i);
                            break;
                        }
                    }
                }
            }

            let mut created_components = Vec::new();
            if let Some(scs_location) = scs_location {
                let mut known_node_categories = Vec::new();
                let scs_export: &NormalExport = &actor_asset.exports[scs_location]
                    .get_normal_export()
                    .expect("Corrupted memory");
                for i in 0..scs_export.properties.len() {
                    let property = &scs_export.properties[i];
                    if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                        if array_property
                            .array_type
                            .as_ref()
                            .map(|e| e.content == "ObjectProperty")
                            .unwrap_or(false)
                            && array_property.name.content == "AllNodes"
                        {
                            for value in &array_property.value {
                                if let Some(object_property) =
                                    cast!(Property, ObjectProperty, value)
                                {
                                    if object_property.value.index > 0 {
                                        known_node_categories.push(object_property.value.index);
                                    }
                                }
                            }
                        }
                    }
                }

                let mut known_parents = HashMap::new();
                for known_node_category in known_node_categories {
                    let known_category: &NormalExport = actor_asset.exports
                        [known_node_category as usize - 1]
                        .get_normal_export()
                        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid export"))?;
                    let is_scs_node = match known_category.base_export.class_index.is_import() {
                        true => {
                            let import = asset
                                .get_import(known_category.base_export.class_index)
                                .ok_or_else(|| {
                                io::Error::new(ErrorKind::Other, "Import not found")
                            })?;
                            import.object_name.content == "SCS_Node"
                        }
                        false => false,
                    };
                    if !is_scs_node {
                        continue;
                    }

                    let mut new_scs = ScsNode {
                        internal_variable_name: String::from("Unknown"),
                        type_link: PackageIndex::new(0),
                        attach_parent: None,
                        original_category: PackageIndex::new(known_node_category),
                    };

                    let mut first_import = None;
                    let mut second_import = None;

                    for property in &known_category.properties {
                        match property.get_name().content.as_str() {
                            "InternalVariableName" => {
                                if let Some(name_property) = cast!(Property, NameProperty, property)
                                {
                                    new_scs.internal_variable_name =
                                        name_property.value.content.clone();
                                }
                            }
                            "ComponentClass" => {
                                if let Some(object_property) =
                                    cast!(Property, ObjectProperty, property)
                                {
                                    let import = actor_asset
                                        .get_import(object_property.value)
                                        .ok_or_else(|| {
                                            io::Error::new(ErrorKind::Other, "No import")
                                        })?
                                        .clone();

                                    second_import = Some(
                                        actor_asset
                                            .get_import(import.outer_index)
                                            .ok_or_else(|| {
                                                io::Error::new(ErrorKind::Other, "No import")
                                            })?
                                            .clone(),
                                    );
                                    first_import = Some(import);
                                }
                            }
                            "ChildNodes" => {
                                if let Some(array_property) =
                                    cast!(Property, ArrayProperty, property)
                                {
                                    if array_property
                                        .array_type
                                        .as_ref()
                                        .map(|e| e.content == "ObjectProperty")
                                        .unwrap_or(false)
                                    {
                                        for value_property in &array_property.value {
                                            if let Some(object_property) =
                                                cast!(Property, ObjectProperty, value_property)
                                            {
                                                known_parents.insert(
                                                    object_property.value.index,
                                                    known_node_category,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if first_import.is_some() && second_import.is_some() {
                        let first_import = first_import.unwrap();
                        let second_import = second_import.unwrap();

                        if asset
                            .find_import(
                                &second_import.class_package,
                                &second_import.class_name,
                                second_import.outer_index,
                                &second_import.object_name,
                            )
                            .is_none()
                        {
                            asset.add_import(second_import);
                        }

                        let type_link = match asset.find_import(
                            &first_import.class_package,
                            &first_import.class_name,
                            first_import.outer_index,
                            &first_import.object_name,
                        ) {
                            Some(e) => PackageIndex::new(e),
                            None => asset.add_import(first_import),
                        };

                        new_scs.type_link = type_link;
                    }
                    created_components.push(new_scs);
                }

                for scs_node in &mut created_components {
                    if let Some(original_category) =
                        known_parents.get(&scs_node.original_category.index)
                    {
                        scs_node.attach_parent = Some(PackageIndex::new(*original_category));
                    }
                }
            }

            let template_category_pointer =
                asset.exports.len() as i32 + created_components.len() as i32 + 1;

            let mut created_component_serialized_list: Vec<Property> = Vec::new();
            let mut attach_parent_correcting = HashMap::new();
            let mut node_name_to_export_index = HashMap::new();
            let mut old_export_to_new_export = HashMap::new();

            for created_component in &created_components {
                let mut scene_export = scene_export.clone();
                scene_export.base_export.class_index = created_component.type_link;
                asset.add_name_reference(created_component.internal_variable_name.clone(), false);
                scene_export.base_export.object_name =
                    FName::new(created_component.internal_variable_name.clone(), 0);
                scene_export.base_export.outer_index = PackageIndex::new(template_category_pointer);

                let mut prop_data: Vec<Property> = Vec::from([
                    BoolProperty {
                        name: FName::from_slice("bNetAddressable"),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: true,
                    }
                    .into(),
                    EnumProperty {
                        name: FName::from_slice("CreationMethod"),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        enum_type: Some(FName::from_slice("EComponentCreationMethod")),
                        value: FName::from_slice(
                            "EComponentCreationMethod::SimpleConstructionScript",
                        ),
                    }
                    .into(),
                ]);

                let mut correction_queue = Vec::new();
                if let Some(attach_parent) = created_component.attach_parent {
                    let next_property = ObjectProperty {
                        name: FName::from_slice("AttachParent"),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: attach_parent,
                    };
                    // todo: correction
                    correction_queue.push(prop_data.len());
                    prop_data.push(next_property.into());
                }

                scene_export.extras = vec![0u8; 4];
                scene_export.properties = prop_data;

                attach_parent_correcting.insert(asset.exports.len(), correction_queue);
                asset.exports.push(scene_export.into());

                created_component_serialized_list.push(
                    ObjectProperty {
                        name: FName::from_slice("BlueprintCreatedComponents"),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: PackageIndex::new(asset.exports.len() as i32),
                    }
                    .into(),
                );

                node_name_to_export_index.insert(
                    created_component.internal_variable_name.clone(),
                    asset.exports.len() as i32,
                );
                old_export_to_new_export.insert(
                    created_component.original_category.index,
                    asset.exports.len() as i32,
                );

                let type_link = asset
                    .get_import(created_component.type_link)
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "No type link"))?;

                let import = Import {
                    class_package: FName::from_slice("/Script/Engine"),
                    class_name: type_link.object_name.clone(),
                    outer_index: actor_template.base_export.class_index,
                    object_name: FName::new(
                        created_component.internal_variable_name.clone() + "_GEN_VARIABLE",
                        0,
                    ),
                };
                asset.add_import(import);
            }

            for (export_index, correction_queue) in attach_parent_correcting {
                let export: &mut NormalExport = asset.exports[export_index]
                    .get_normal_export_mut()
                    .expect("Corrupted memory");
                for correction in correction_queue {
                    let property =
                        cast!(Property, ObjectProperty, &mut export.properties[correction])
                            .expect("Corrupted memory");
                    property.value = PackageIndex::new(
                        *old_export_to_new_export
                            .get(&property.value.index)
                            .ok_or_else(|| {
                                io::Error::new(ErrorKind::Other, "No correction data")
                            })?,
                    );
                }
            }

            let mut determined_prop_data: Vec<Property> = Vec::from([
                BoolProperty {
                    name: FName::from_slice("bHidden"),
                    property_guid: Some([0u8; 16]),
                    duplication_index: 0,
                    value: true,
                }
                .into(),
                ArrayProperty::from_arr(
                    FName::from_slice("BlueprintCreatedComponents"),
                    Some(FName::from_slice("ObjectProperty")),
                    created_component_serialized_list,
                )
                .into(),
            ]);

            for (node_name, export_index) in node_name_to_export_index {
                if node_name == "DefaultSceneRoot" {
                    determined_prop_data.push(
                        ObjectProperty {
                            name: FName::from_slice("RootComponent"),
                            property_guid: Some([0u8; 16]),
                            duplication_index: 0,
                            value: PackageIndex::new(export_index),
                        }
                        .into(),
                    );
                }
                determined_prop_data.push(
                    ObjectProperty {
                        name: FName::new(node_name, 0),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: PackageIndex::new(export_index),
                    }
                    .into(),
                );
            }

            actor_template
                .base_export
                .serialization_before_create_dependencies
                .push(blueprint_generated_class_import);
            actor_template
                .base_export
                .serialization_before_create_dependencies
                .push(default_import);
            actor_template
                .base_export
                .create_before_create_dependencies
                .push(PackageIndex::new(level_export_index as i32 + 1));
            actor_template.extras = vec![0u8; 4];
            actor_template.properties = determined_prop_data;
            asset.exports.push(actor_template.into());

            let exports_len = asset.exports.len();
            let level_export = cast!(Export, LevelExport, &mut asset.exports[level_export_index])
                .expect("Corrupted memory");
            level_export.index_data.push(exports_len as i32);
            level_export
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .push(PackageIndex::new(exports_len as i32));
        }

        write_asset(integrated_pak, &asset, &String::from(map_path))
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
