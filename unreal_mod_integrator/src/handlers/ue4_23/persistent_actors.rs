use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Cursor, ErrorKind};
use std::path::Path;

use unreal_asset::engine_version::EngineVersion;
use unreal_asset::reader::archive_trait::ArchiveTrait;
use unreal_asset::unversioned::ancestry::Ancestry;
use unreal_asset::{
    cast,
    exports::{normal_export::NormalExport, Export, ExportBaseTrait, ExportNormalTrait},
    properties::{
        array_property::ArrayProperty, enum_property::EnumProperty, int_property::BoolProperty,
        object_property::ObjectProperty, Property, PropertyDataTrait,
    },
    types::PackageIndex,
    Asset, Import,
};
use unreal_pak::{PakMemory, PakReader};

use crate::helpers::{get_asset, write_asset};
use crate::Error;

const LEVEL_TEMPLATE_ASSET: &[u8] = include_bytes!("assets/LevelTemplate.umap");

#[derive(Default)]
struct ScsNode {
    internal_variable_name: String,
    type_link: PackageIndex,
    attach_parent: Option<PackageIndex>,
    original_category: PackageIndex,
}

#[allow(clippy::ptr_arg)]
pub fn handle_persistent_actors(
    game_name: &'static str,
    map_paths: &[&str],
    integrated_pak: &mut PakMemory,
    game_paks: &mut Vec<PakReader<File>>,
    mod_paks: &mut Vec<PakReader<File>>,
    persistent_actor_arrays: &Vec<serde_json::Value>,
) -> Result<(), Error> {
    let level_asset = Asset::new(
        Cursor::new(LEVEL_TEMPLATE_ASSET),
        None,
        EngineVersion::VER_UE4_23,
    )
    .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let actor_template = cast!(
        Export,
        NormalExport,
        level_asset.asset_data.exports[2].clone()
    )
    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted actor_template"))?;

    let scene_export = cast!(
        Export,
        NormalExport,
        level_asset.asset_data.exports[11].clone()
    )
    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted scene_component"))?;

    let mut persistent_actors = Vec::new();
    for persistent_actors_array in persistent_actor_arrays {
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

    for map_path in map_paths {
        let mut asset = get_asset(
            integrated_pak,
            game_paks,
            mod_paks,
            &map_path.to_string(),
            EngineVersion::VER_UE4_23,
        )?;

        let mut level_export_index = None;
        for i in 0..asset.asset_data.exports.len() {
            if cast!(Export, LevelExport, &asset.asset_data.exports[i]).is_some() {
                level_export_index = Some(i);
                break;
            }
        }
        let level_export_index = level_export_index
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No level export"))?;

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
            actor_template.base_export.object_name = asset.add_fname(component);
            actor_template.base_export.template_index = default_import;
            actor_template.base_export.outer_index =
                PackageIndex::new(level_export_index as i32 + 1); // package index starts from 1

            let actor_asset_path = unreal_helpers::game_to_absolute(game_name, &component_path_raw)
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid actor path"))?;

            let actor_asset = get_asset(
                integrated_pak,
                game_paks,
                mod_paks,
                &actor_asset_path,
                EngineVersion::VER_UE4_23,
            )?;

            let mut scs_location = None;
            for i in 0..actor_asset.asset_data.exports.len() {
                let export = &actor_asset.asset_data.exports[i];
                if let Some(normal_export) = export.get_normal_export() {
                    if normal_export.base_export.class_index.is_import() {
                        let import = asset
                            .get_import(normal_export.base_export.class_index)
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Import not found"))?;
                        if import.object_name.get_content() == "SimpleConstructionScript" {
                            scs_location = Some(i);
                            break;
                        }
                    }
                }
            }

            let mut created_components = Vec::new();
            if let Some(scs_location) = scs_location {
                let mut known_node_categories = Vec::new();
                let scs_export: &NormalExport = actor_asset.asset_data.exports[scs_location]
                    .get_normal_export()
                    .expect("Corrupted memory");
                for i in 0..scs_export.properties.len() {
                    let property = &scs_export.properties[i];
                    if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                        if array_property
                            .array_type
                            .as_ref()
                            .map(|e| e.get_content() == "ObjectProperty")
                            .unwrap_or(false)
                            && array_property.name.get_content() == "AllNodes"
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
                    let known_category: &NormalExport = actor_asset.asset_data.exports
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
                            import.object_name.get_content() == "SCS_Node"
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
                        match property.get_name().get_content().as_str() {
                            "InternalVariableName" => {
                                if let Some(name_property) = cast!(Property, NameProperty, property)
                                {
                                    new_scs.internal_variable_name =
                                        name_property.value.get_content();
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
                                        .map(|e| e.get_content() == "ObjectProperty")
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

                    if let (Some(first_import), Some(second_import)) = (first_import, second_import)
                    {
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
                asset.asset_data.exports.len() as i32 + created_components.len() as i32 + 1;

            let mut created_component_serialized_list: Vec<Property> = Vec::new();
            let mut attach_parent_correcting = HashMap::new();
            let mut node_name_to_export_index = HashMap::new();
            let mut old_export_to_new_export = HashMap::new();

            for created_component in &created_components {
                let mut scene_export = scene_export.clone();
                scene_export.base_export.class_index = created_component.type_link;
                scene_export.base_export.object_name =
                    asset.add_fname(&created_component.internal_variable_name);
                scene_export.base_export.outer_index = PackageIndex::new(template_category_pointer);

                let mut prop_data: Vec<Property> = Vec::from([
                    BoolProperty {
                        name: asset.add_fname("bNetAddressable"),
                        ancestry: Ancestry::default(),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: true,
                    }
                    .into(),
                    EnumProperty {
                        name: asset.add_fname("CreationMethod"),
                        ancestry: Ancestry::default(),
                        inner_type: None,
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        enum_type: Some(asset.add_fname("EComponentCreationMethod")),
                        value: asset
                            .add_fname("EComponentCreationMethod::SimpleConstructionScript"),
                    }
                    .into(),
                ]);

                let mut correction_queue = Vec::new();
                if let Some(attach_parent) = created_component.attach_parent {
                    let next_property = ObjectProperty {
                        name: asset.add_fname("AttachParent"),
                        ancestry: Ancestry::default(),
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

                attach_parent_correcting.insert(asset.asset_data.exports.len(), correction_queue);
                asset.asset_data.exports.push(scene_export.into());

                created_component_serialized_list.push(
                    ObjectProperty {
                        name: asset.add_fname("BlueprintCreatedComponents"),
                        ancestry: Ancestry::default(),
                        property_guid: Some([0u8; 16]),
                        duplication_index: 0,
                        value: PackageIndex::new(asset.asset_data.exports.len() as i32),
                    }
                    .into(),
                );

                node_name_to_export_index.insert(
                    created_component.internal_variable_name.clone(),
                    asset.asset_data.exports.len() as i32,
                );
                old_export_to_new_export.insert(
                    created_component.original_category.index,
                    asset.asset_data.exports.len() as i32,
                );

                let type_link = asset
                    .get_import(created_component.type_link)
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "No type link"))?;

                let import = Import {
                    class_package: asset.add_fname("/Script/Engine"),
                    class_name: type_link.object_name.clone(),
                    outer_index: actor_template.base_export.class_index,
                    object_name: asset.add_fname(
                        &(created_component.internal_variable_name.clone() + "_GEN_VARIABLE"),
                    ),
                };
                asset.add_import(import);
            }

            for (export_index, correction_queue) in attach_parent_correcting {
                let export: &mut NormalExport = asset.asset_data.exports[export_index]
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
                    name: asset.add_fname("bHidden"),
                    ancestry: Ancestry::default(),
                    property_guid: Some([0u8; 16]),
                    duplication_index: 0,
                    value: true,
                }
                .into(),
                ArrayProperty::from_arr(
                    asset.add_fname("BlueprintCreatedComponents"),
                    Ancestry::default(),
                    Some(asset.add_fname("ObjectProperty")),
                    created_component_serialized_list,
                )
                .into(),
            ]);

            for (node_name, export_index) in node_name_to_export_index {
                if node_name == "DefaultSceneRoot" {
                    determined_prop_data.push(
                        ObjectProperty {
                            name: asset.add_fname("RootComponent"),
                            ancestry: Ancestry::default(),
                            property_guid: Some([0u8; 16]),
                            duplication_index: 0,
                            value: PackageIndex::new(export_index),
                        }
                        .into(),
                    );
                }
                determined_prop_data.push(
                    ObjectProperty {
                        name: asset.add_fname(&node_name),
                        ancestry: Ancestry::default(),
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
            asset.asset_data.exports.push(actor_template.into());

            let exports_len = PackageIndex::new(asset.asset_data.exports.len() as i32);
            let level_export = cast!(
                Export,
                LevelExport,
                &mut asset.asset_data.exports[level_export_index]
            )
            .expect("Corrupted memory");
            level_export.actors.push(exports_len);
            level_export
                .get_base_export_mut()
                .create_before_serialization_dependencies
                .push(exports_len);
        }

        write_asset(integrated_pak, &asset, &map_path.to_string())
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
