use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::{Export, ExportBaseTrait, ExportNormalTrait},
    flags::EObjectFlags,
    properties::{
        guid_property::GuidProperty, int_property::BoolProperty, object_property::ObjectProperty,
        str_property::NameProperty, struct_property::StructProperty, Property, PropertyDataTrait,
    },
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    uproperty::UProperty,
    Asset, Import,
};
use unreal_modintegrator::write_asset;
use unreal_pak::PakFile;
use uuid::Uuid;

use crate::assets::{ACTOR_TEMPLATE_ASSET, ACTOR_TEMPLATE_EXPORT};

use super::{game_to_absolute, get_asset};

#[allow(clippy::ptr_arg)]
pub(crate) fn handle_linked_actor_components(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    _mod_paks: &mut Vec<PakFile>,
    linked_actors_maps: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut actor_asset = Asset::new(
        ACTOR_TEMPLATE_ASSET.to_vec(),
        Some(ACTOR_TEMPLATE_EXPORT.to_vec()),
    );
    actor_asset.engine_version = VER_UE4_23;
    actor_asset
        .parse_data()
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let gen_variable =
        cast!(Export, NormalExport, &actor_asset.exports[0]).expect("Corrupted ActorTemplate");
    let component_export =
        cast!(Export, PropertyExport, &actor_asset.exports[1]).expect("Corrupted ActorTemplate");
    let scs_export =
        cast!(Export, NormalExport, &actor_asset.exports[2]).expect("Corrupted ActorTemplate");

    let mut new_components = HashMap::new();

    for linked_actor_map in &linked_actors_maps {
        let linked_actors_map = linked_actor_map
            .as_object()
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid linked_actor_components"))?;
        for (name, components) in linked_actors_map.iter() {
            let components = components.as_array().ok_or_else(|| {
                io::Error::new(ErrorKind::Other, "Invalid linked_actor_components2")
            })?;

            let entry = new_components.entry(name.clone()).or_insert_with(Vec::new);
            for component in components {
                let component_name = component.as_str().ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "Invalid linked_actor_components3")
                })?;
                entry.push(String::from(component_name));
            }
        }
    }

    for (name, components) in &new_components {
        let name = game_to_absolute(name)
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid asset name"))?;
        let mut asset = get_asset(integrated_pak, game_paks, &name, VER_UE4_23)?;

        for component_path_raw in components {
            let mut actor_index = None;
            let mut simple_construction_script = None;
            let mut cdo_location = None;
            for i in 0..asset.exports.len() {
                let export = &asset.exports[i];
                if let Some(normal_export) = export.get_normal_export() {
                    if normal_export.base_export.class_index.is_import() {
                        let import = asset
                            .get_import(normal_export.base_export.class_index)
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Import not found"))?;
                        match import.object_name.content.as_str() {
                            "BlueprintGeneratedClass" => actor_index = Some(i),
                            "SimpleConstructionScript" => simple_construction_script = Some(i),
                            _ => {}
                        }
                    }
                    if (EObjectFlags::RF_CLASS_DEFAULT_OBJECT
                        & EObjectFlags::from_bits(normal_export.base_export.object_flags)
                            .ok_or_else(|| {
                                io::Error::new(ErrorKind::Other, "Invalid object flags")
                            })?)
                        == EObjectFlags::RF_CLASS_DEFAULT_OBJECT
                    {
                        cdo_location = Some(i);
                    }
                }
            }

            let actor_index =
                actor_index.ok_or_else(|| io::Error::new(ErrorKind::Other, "Actor not found"))?;
            let actor = actor_index as i32 + 1;
            let simple_construction_script_index = simple_construction_script
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "SCS not found"))?;
            let simple_construction_script = simple_construction_script_index as i32 + 1;
            let cdo_location =
                cdo_location.ok_or_else(|| io::Error::new(ErrorKind::Other, "CDO not found"))?;

            let class_object_property_import = asset
                .find_import_no_index(
                    &FName::from_slice("/Script/CoreUObject"),
                    &FName::from_slice("Class"),
                    &FName::from_slice("ObjectProperty"),
                )
                .expect("No class object property import");

            let default_object_property_import = asset
                .find_import_no_index(
                    &FName::from_slice("/Script/CoreUObject"),
                    &FName::from_slice("ObjectProperty"),
                    &FName::from_slice("Default__ObjectProperty"),
                )
                .expect("No default objectproperty");

            let scs_node_import = asset
                .find_import_no_index(
                    &FName::from_slice("/Script/CoreUObject"),
                    &FName::from_slice("Class"),
                    &FName::from_slice("SCS_Node"),
                )
                .expect("No SCS_Node");

            let default_scs_node_import = asset
                .find_import_no_index(
                    &FName::from_slice("/Script/Engine"),
                    &FName::from_slice("SCS_Node"),
                    &FName::from_slice("Default__SCS_Node"),
                )
                .expect("No default scs");

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
            let component_c = String::from(component) + "_C";
            let default_component = String::from("Default__") + component + "_C";

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
                object_name: asset.add_fname(&component_c),
            };
            let blueprint_generated_class_import =
                asset.add_import(blueprint_generated_class_import);

            let default_import = Import {
                class_package: asset.add_fname("/Game/AddMe"),
                class_name: asset.add_fname(&component_c),
                outer_index: package_import,
                object_name: asset.add_fname(&default_component),
            };
            let default_import = asset.add_import(default_import);

            let mut component_export = component_export.clone();
            let component_object_property =
                cast!(UProperty, UObjectProperty, &mut component_export.property)
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted starter pak"))?;
            component_object_property.property_class = blueprint_generated_class_import;

            let component_base_export = component_export.get_base_export_mut();
            component_base_export.object_name = asset.add_fname(component);
            component_base_export.create_before_serialization_dependencies =
                Vec::from([blueprint_generated_class_import]);
            component_base_export.create_before_create_dependencies =
                Vec::from([PackageIndex::new(actor)]);
            component_base_export.outer_index = PackageIndex::new(actor);
            component_base_export.class_index = PackageIndex::new(class_object_property_import);
            component_base_export.template_index =
                PackageIndex::new(default_object_property_import);

            asset.exports.push(component_export.into());

            let component_export_index = asset.exports.len() as i32;
            let actor_export = cast!(Export, ClassExport, &mut asset.exports[actor_index])
                .expect("Corrupted memory");
            actor_export
                .struct_export
                .children
                .push(PackageIndex::new(component_export_index));
            actor_export
                .struct_export
                .normal_export
                .base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(component_export_index));

            let mut component_gen_variable = gen_variable.clone();
            let mut component_gen_variable_base_export =
                component_gen_variable.get_base_export_mut();
            component_gen_variable_base_export.outer_index = PackageIndex::new(actor);
            component_gen_variable_base_export.class_index = blueprint_generated_class_import;
            component_gen_variable_base_export.template_index = default_import;
            component_gen_variable_base_export.serialization_before_serialization_dependencies =
                Vec::from([PackageIndex::new(actor)]);
            component_gen_variable_base_export.serialization_before_create_dependencies =
                Vec::from([blueprint_generated_class_import, default_import]);
            component_gen_variable_base_export.create_before_create_dependencies =
                Vec::from([PackageIndex::new(actor)]);
            component_gen_variable_base_export.object_name =
                asset.add_fname(&(String::from(component) + "_GEN_VARIABLE"));

            let mut component_gen_variable_normal_export =
                component_gen_variable.get_normal_export_mut().unwrap();
            asset.add_fname("BoolProperty");
            component_gen_variable_normal_export.properties = Vec::from([BoolProperty {
                name: asset.add_fname("bAutoActivate"),
                property_guid: Some([0u8; 16]),
                duplication_index: 0,
                value: true,
            }
            .into()]);

            asset.exports.push(component_gen_variable.into());
            let component_gen_variable_index = asset.exports.len() as i32;

            let mut scs_node = scs_export.clone();
            let scs_node_normal_export = scs_node
                .get_normal_export_mut()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Corrupted starter pak"))?;
            scs_node_normal_export.properties = Vec::from([
                ObjectProperty {
                    name: asset.add_fname("ComponentClass"),
                    property_guid: Some([0u8; 16]),
                    duplication_index: 0,
                    value: blueprint_generated_class_import,
                }
                .into(),
                ObjectProperty {
                    name: asset.add_fname("ComponentTemplate"),
                    property_guid: Some([0u8; 16]),
                    duplication_index: 0,
                    value: PackageIndex::new(component_gen_variable_index),
                }
                .into(),
                StructProperty {
                    name: asset.add_fname("VariableGuid"),
                    struct_type: Some(asset.add_fname("Guid")),
                    struct_guid: Some([0u8; 16]),
                    property_guid: None,
                    duplication_index: 0,
                    serialize_none: true,
                    value: Vec::from([GuidProperty {
                        name: asset.add_fname("VariableGuid"),
                        property_guid: None,
                        duplication_index: 0,
                        value: Uuid::new_v4().into_bytes(),
                    }
                    .into()]),
                }
                .into(),
                NameProperty {
                    name: asset.add_fname("InternalVariableName"),
                    property_guid: None,
                    duplication_index: 0,
                    value: asset.add_fname(component),
                }
                .into(),
            ]);
            scs_node_normal_export.base_export.outer_index =
                PackageIndex::new(simple_construction_script);
            scs_node_normal_export.base_export.class_index = PackageIndex::new(scs_node_import);
            scs_node_normal_export.base_export.template_index =
                PackageIndex::new(default_scs_node_import);
            scs_node_normal_export
                .base_export
                .create_before_serialization_dependencies = Vec::from([
                blueprint_generated_class_import,
                PackageIndex::new(component_gen_variable_index),
            ]);
            scs_node_normal_export
                .base_export
                .serialization_before_create_dependencies = Vec::from([
                PackageIndex::new(scs_node_import),
                PackageIndex::new(default_scs_node_import),
            ]);
            scs_node_normal_export
                .base_export
                .create_before_create_dependencies =
                Vec::from([PackageIndex::new(simple_construction_script)]);

            let mut last_scs_node_index = 0;
            for export in &asset.exports {
                let object_name = &export.get_base_export().object_name;
                if object_name.content == "SCS_Node" && last_scs_node_index < object_name.index {
                    last_scs_node_index = object_name.index;
                }
            }
            scs_node_normal_export.base_export.object_name =
                FName::new("SCS_Node".to_string(), last_scs_node_index + 1);

            asset.exports.push(scs_node.into());
            let scs_node_index = asset.exports.len() as i32;

            let cdo_base_export = asset.exports[cdo_location].get_base_export_mut();
            cdo_base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(scs_node_index));
            cdo_base_export
                .serialization_before_serialization_dependencies
                .push(PackageIndex::new(component_gen_variable_index));

            let simple_construction_script_export = asset.exports[simple_construction_script_index]
                .get_normal_export_mut()
                .expect("Corrupted memory");
            simple_construction_script_export
                .base_export
                .create_before_serialization_dependencies
                .push(PackageIndex::new(scs_node_index));

            for property in &mut simple_construction_script_export.properties {
                if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                    let name = array_property.name.content.as_str();
                    if name == "AllNodes" || name == "RootNodes" {
                        let mut last_index = 0;
                        for property in &array_property.value {
                            let index = property.get_name().index;
                            if last_index < index {
                                last_index = index;
                            }
                        }

                        array_property.value.push(
                            ObjectProperty {
                                name: FName::new((last_index + 1).to_string(), -2147483648),
                                property_guid: None,
                                duplication_index: 0,
                                value: PackageIndex::new(scs_node_index),
                            }
                            .into(),
                        );
                    }
                }
            }
        }

        write_asset(integrated_pak, &asset, &name)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
