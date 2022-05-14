use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::{Export, ExportNormalTrait},
    properties::{
        array_property::ArrayProperty, enum_property::EnumProperty, int_property::BoolProperty,
        object_property::ObjectProperty, Property, PropertyDataTrait,
    },
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Asset, Import,
};
use unreal_modintegrator::{find_asset, read_asset, write_asset};
use unreal_pak::PakFile;

use super::{game_to_absolute, get_asset, MAP_PATHS};

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
    persistent_actor_arrays: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut level_asset = Asset::new(crate::assets::LEVEL_TEMPLATE_ASSET.to_vec(), None);
    level_asset.engine_version = VER_UE4_23;
    level_asset
        .parse_data()
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let actor_template = level_asset
        .get_export(PackageIndex::new(2))
        .and_then(|e| {
            if let Export::NormalExport(e) = e {
                Some(e)
            } else {
                None
            }
        })
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted actor_template"))?;

    let scene_component = level_asset
        .get_export(PackageIndex::new(11))
        .and_then(|e| match e {
            Export::NormalExport(e) => Some(e),
            _ => None,
        })
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted scene_component"))?;

    for map_path in MAP_PATHS {
        let mut asset = get_asset(
            integrated_pak,
            game_paks,
            &String::from(map_path),
            VER_UE4_23,
        )?;

        let mut level_index = None;
        for i in 0..asset.exports.len() {
            let export = &asset.exports[i];
            if let Export::LevelExport(_) = export {
                level_index = Some(i);
                break;
            }
        }
        if level_index.is_none() {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Unable to find Level category",
            ));
        }
        let level_index = level_index.unwrap();

        asset.add_fname("bHidden");
        asset.add_fname("bNetAddressable");
        asset.add_fname("CreationMethod");
        asset.add_fname("EComponentCreationMethod");
        asset.add_fname("EComponentCreationMethod::SimpleConstructionScript");
        asset.add_fname("BlueprintCreatedComponents");
        asset.add_fname("AttachParent");
        asset.add_fname("RootComponent");

        for persistent_actors in &persistent_actor_arrays {
            let persistent_actors = persistent_actors
                .as_array()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?;

            for persistent_actor in persistent_actors {
                let actor_path_raw = persistent_actor
                    .as_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?;
                let actor = Path::new(actor_path_raw)
                    .file_stem()
                    .and_then(|e| e.to_str())
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid persistent actors"))?;

                let (actor_path_raw, actor) = match actor.contains('.') {
                    true => {
                        let split: Vec<&str> = actor.split('.').collect();
                        (split[0], &split[1][..split[1].len() - 2])
                    }
                    false => (actor_path_raw, actor),
                };

                asset.add_fname(actor_path_raw);
                asset.add_fname(&(String::from(actor) + "_C"));
                asset.add_fname(&(String::from("Default__") + actor + "_C"));
                asset.add_fname(actor);

                let first_import = Import {
                    class_package: FName::from_slice("/Script/CoreUObject"),
                    class_name: FName::from_slice("Package"),
                    outer_index: PackageIndex::new(0),
                    object_name: FName::from_slice(actor_path_raw),
                };
                let first_import = asset.add_import(first_import);

                let blueprint_import = Import {
                    class_package: FName::from_slice("/Script/Engine"),
                    class_name: FName::from_slice("BlueprintGeneratedClass"),
                    outer_index: first_import,
                    object_name: FName::new(String::from(actor) + "_C", 0),
                };
                let blueprint_import = asset.add_import(blueprint_import);

                let component_import = Import {
                    class_package: FName::from_slice(actor_path_raw),
                    class_name: FName::new(String::from(actor) + "_C", 0),
                    outer_index: blueprint_import,
                    object_name: FName::new(String::from("Default__") + actor + "_C", 0),
                };
                let component_import = asset.add_import(component_import);

                let mut actor_template = actor_template.clone();
                actor_template.base_export.class_index = blueprint_import;
                actor_template.base_export.object_name = FName::from_slice(actor);
                actor_template.base_export.template_index = component_import;
                actor_template.base_export.outer_index = PackageIndex::new(level_index as i32 + 1);

                let mut all_blueprint_created_components = Vec::new();

                let asset_name = game_to_absolute(actor).ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Invalid persistent actor path")
                })?;

                let game_asset = find_asset(game_paks, &asset_name)
                    .and_then(|e| read_asset(&mut game_paks[e], VER_UE4_23, &asset_name).ok());
                if let Some(game_asset) = game_asset {
                    let mut scs_export = None;

                    for i in 0..game_asset.exports.len() {
                        let export = &game_asset.exports[i];
                        if let Export::NormalExport(normal_export) = export {
                            if normal_export.base_export.class_index.is_import() {
                                let is_scs = game_asset
                                    .get_import(normal_export.base_export.class_index)
                                    .map(|e| &e.object_name.content == "SimpleConstructionScript")
                                    .unwrap_or(false);

                                if is_scs {
                                    scs_export = Some(normal_export);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(scs_export) = scs_export {
                        let mut known_node_categories = Vec::new();

                        for property in &scs_export.properties {
                            if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                                if array_property
                                    .array_type
                                    .as_ref()
                                    .map(|e| e.content == "ObjectProperty")
                                    .unwrap_or(false)
                                    && array_property.name.content == "AllNodes"
                                {
                                    for array_element in &array_property.value {
                                        if let Some(object_property) =
                                            cast!(Property, ObjectProperty, array_element)
                                        {
                                            if object_property.value.index > 0 {
                                                known_node_categories.push(object_property.value);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let mut known_parents = HashMap::new();
                        for known_node_category in known_node_categories {
                            let known_category =
                                &game_asset.exports[known_node_category.index as usize - 1];
                            if let Some(known_normal_category) = known_category.get_normal_export()
                            {
                                let is_scs_node = match known_normal_category
                                    .base_export
                                    .class_index
                                    .is_import()
                                {
                                    true => game_asset
                                        .get_import(known_normal_category.base_export.class_index)
                                        .map(|e| e.object_name.content == "SCS_Node")
                                        .unwrap_or(false),
                                    false => false,
                                };
                                if !is_scs_node {
                                    continue;
                                }

                                let mut new_scs = ScsNode {
                                    internal_variable_name: String::from("Unknown"),
                                    original_category: known_node_category,
                                    ..Default::default()
                                };

                                let mut import_1 = None;
                                let mut import_2 = None;

                                for property in &known_normal_category.properties {
                                    match property.get_name().content.as_str() {
                                        "InternalVariableName" => {
                                            if let Some(name_property) =
                                                cast!(Property, NameProperty, property)
                                            {
                                                new_scs.internal_variable_name =
                                                    name_property.value.content.clone();
                                            }
                                        }
                                        "ComponentClass" => {
                                            if let Some(object_property) =
                                                cast!(Property, ObjectProperty, property)
                                            {
                                                let import = game_asset
                                                    .get_import(object_property.value)
                                                    .ok_or_else(|| {
                                                        io::Error::new(
                                                            ErrorKind::Other,
                                                            "No such link",
                                                        )
                                                    })?;
                                                import_1 = Some(import);
                                                import_2 = Some(
                                                    game_asset
                                                        .get_import(import.outer_index)
                                                        .ok_or_else(|| {
                                                            io::Error::new(
                                                                ErrorKind::Other,
                                                                "No such link",
                                                            )
                                                        })?,
                                                );
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
                                                    for property in &array_property.value {
                                                        if let Some(object_property) = cast!(
                                                            Property,
                                                            ObjectProperty,
                                                            property
                                                        ) {
                                                            known_parents.insert(
                                                                object_property.value,
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

                                if let (Some(import_1), Some(import_2)) = (import_1, import_2) {
                                    let added_import = asset.find_import(
                                        &import_2.class_package,
                                        &import_2.class_name,
                                        import_2.outer_index,
                                        &import_2.object_name,
                                    );
                                    if let Some(_added_import) = added_import {
                                        asset.add_import(import_2.clone());
                                    }

                                    let new_type_import = asset.find_import(
                                        &import_1.class_package,
                                        &import_1.class_name,
                                        import_1.outer_index,
                                        &import_1.object_name,
                                    );
                                    let new_type_import = match new_type_import {
                                        Some(_) => asset.add_import(import_1.clone()),
                                        None => PackageIndex::new(0),
                                    };
                                    new_scs.type_link = new_type_import;
                                }

                                all_blueprint_created_components.push(new_scs);
                            }
                        }

                        for node in &mut all_blueprint_created_components {
                            if let Some(parent) = known_parents.get(&node.original_category) {
                                node.attach_parent = Some(*parent);
                            }
                        }
                    }
                }

                let template_category_pointer =
                    (asset.exports.len() + all_blueprint_created_components.len() + 1) as i32;

                let mut serialized_blueprint_created_components: Vec<Property> = Vec::new();
                let mut scene_exports: Vec<Export> = Vec::new();

                let mut node_name_to_export_index = HashMap::new();
                let mut old_export_to_new_export = HashMap::new();

                for blueprint_created_component in &all_blueprint_created_components {
                    let mut scene_export = scene_component.clone();

                    scene_export.base_export.class_index = blueprint_created_component.type_link;
                    asset.add_name_reference(
                        blueprint_created_component.internal_variable_name.clone(),
                        false,
                    );
                    scene_export.base_export.object_name = FName::new(
                        blueprint_created_component.internal_variable_name.clone(),
                        0,
                    );
                    scene_export.base_export.outer_index =
                        PackageIndex::new(template_category_pointer);

                    let mut prop_data: Vec<Property> = Vec::from([
                        BoolProperty {
                            name: FName::from_slice("bNetAddressable"),
                            property_guid: None,
                            duplication_index: 0,
                            value: true,
                        }
                        .into(),
                        EnumProperty {
                            name: FName::from_slice("CreationMethod"),
                            property_guid: None,
                            duplication_index: 0,
                            enum_type: Some(FName::from_slice("EComponentCreationMethod")),
                            value: FName::from_slice(
                                "EComponentCreationMode::SimpleConstructionScript",
                            ),
                        }
                        .into(),
                    ]);

                    if let Some(attach_parent) = blueprint_created_component.attach_parent {
                        let next_parent = ObjectProperty {
                            name: FName::from_slice("AttachParent"),
                            property_guid: None,
                            duplication_index: 0,
                            value: attach_parent,
                        };
                        prop_data.push(next_parent.into());
                    }

                    scene_export.extras = [0u8; 4].to_vec();
                    scene_export.properties = prop_data;
                    scene_exports.push(scene_export.into());

                    let count = (asset.exports.len() + scene_exports.len()) as i32;
                    serialized_blueprint_created_components.push(
                        ObjectProperty {
                            name: FName::from_slice("BlueprintCreatedComponents"),
                            property_guid: None,
                            duplication_index: 0,
                            value: PackageIndex::new(count),
                        }
                        .into(),
                    );
                    node_name_to_export_index.insert(
                        blueprint_created_component.internal_variable_name.clone(),
                        count,
                    );
                    old_export_to_new_export
                        .insert(blueprint_created_component.original_category, count);

                    let import = Import {
                        class_package: FName::from_slice("/Script/Engine"),
                        class_name: asset
                            .get_import(blueprint_created_component.type_link)
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such import"))?
                            .object_name
                            .clone(),
                        outer_index: actor_template.base_export.class_index,
                        object_name: FName::new(
                            blueprint_created_component.internal_variable_name.clone()
                                + "_GEN_VARIABLE",
                            0,
                        ),
                    };
                    asset.add_import(import);
                }

                for export in &mut scene_exports {
                    if let Some(normal_export) = export.get_normal_export_mut() {
                        for property in &mut normal_export.properties {
                            if let Some(object_property) = cast!(Property, ObjectProperty, property)
                            {
                                if object_property.name.content == "AttachParent" {
                                    object_property.value = PackageIndex::new(
                                        old_export_to_new_export[&object_property.value],
                                    );
                                }
                            }
                        }
                    }
                }

                asset.exports.extend(scene_exports);

                let mut template_prop_data: Vec<Property> = Vec::from([BoolProperty {
                    name: FName::from_slice("bHidden"),
                    property_guid: None,
                    duplication_index: 0,
                    value: true,
                }
                .into()]);

                let mut array_template_prop = ArrayProperty::default();
                array_template_prop.name = FName::from_slice("BlueprintCreatedComponents");
                array_template_prop.array_type = Some(FName::from_slice("ObjectProperty"));
                array_template_prop.value = serialized_blueprint_created_components;
                template_prop_data.push(array_template_prop.into());

                for (node_name, export_index) in node_name_to_export_index {
                    if node_name == "DefaultSceneRoot" {
                        template_prop_data.push(
                            ObjectProperty {
                                name: FName::from_slice("RootComponent"),
                                property_guid: None,
                                duplication_index: 0,
                                value: PackageIndex::new(export_index),
                            }
                            .into(),
                        );
                    }
                    template_prop_data.push(
                        ObjectProperty {
                            name: FName::new(node_name, 0),
                            property_guid: None,
                            duplication_index: 0,
                            value: PackageIndex::new(export_index),
                        }
                        .into(),
                    );
                }

                actor_template
                    .base_export
                    .serialization_before_create_dependencies
                    .push(blueprint_import);
                actor_template
                    .base_export
                    .serialization_before_create_dependencies
                    .push(component_import);
                actor_template
                    .base_export
                    .create_before_create_dependencies
                    .push(PackageIndex::new(level_index as i32 + 1));
                actor_template.extras = [0u8; 4].to_vec();
                actor_template.properties = template_prop_data;
                asset.exports.push(actor_template.into());

                let len = asset.exports.len() as i32;
                let level_export = &mut asset.exports[level_index];
                let level_export =
                    cast!(Export, LevelExport, level_export).expect("Corrupted memory");
                level_export.index_data.push(len);
                level_export
                    .normal_export
                    .base_export
                    .create_before_serialization_dependencies
                    .push(PackageIndex::new(len));
            }
        }

        write_asset(integrated_pak, &asset, &String::from(map_path))
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
