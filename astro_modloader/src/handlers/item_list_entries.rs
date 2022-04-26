use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::Export,
    properties::{
        object_property::{ObjectProperty, SoftObjectProperty},
        Property,
    },
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Import,
};
use unreal_modintegrator::write_asset;
use unreal_pak::PakFile;

use super::{game_to_absolute, get_asset};

pub(crate) fn handle_item_list_entries(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    item_list_entires_maps: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut new_items = HashMap::new();

    for item_list_entries_map in &item_list_entires_maps {
        let item_list_entries_map = item_list_entries_map.as_object().ok_or(io::Error::new(
            ErrorKind::Other,
            "Invalid item_list_entries",
        ))?;

        for (name, item_list_entries) in item_list_entries_map {
            let item_list_entries = item_list_entries.as_object().ok_or(io::Error::new(
                ErrorKind::Other,
                "Invalid item_list_entries",
            ))?;
            let new_items_entry = new_items
                .entry(name.clone())
                .or_insert_with(|| HashMap::new());

            for (item_name, entries) in item_list_entries {
                let entries = entries.as_array().ok_or(io::Error::new(
                    ErrorKind::Other,
                    "Invalid item_list_entries",
                ))?;

                let new_items_entry_map = new_items_entry
                    .entry(item_name.clone())
                    .or_insert_with(|| Vec::new());
                for entry in entries {
                    let entry = entry.as_str().ok_or(io::Error::new(
                        ErrorKind::Other,
                        "Invalid item_list_entries",
                    ))?;
                    new_items_entry_map.push(String::from(entry));
                }
            }
        }
    }

    for (name, entries) in &new_items {
        let name = game_to_absolute(&name)
            .ok_or(io::Error::new(ErrorKind::Other, "Invalid asset name"))?;
        let mut asset = get_asset(integrated_pak, game_paks, &name, VER_UE4_23)?;
        let mut item_types_property = HashMap::new();

        for i in 0..asset.exports.len() {
            let export = &asset.exports[i];
            if let Some(normal_export) = cast!(Export, NormalExport, export) {
                for property in &normal_export.properties {
                    for (name, _) in &new_items {
                        let arr_name = match name.contains(".") {
                            true => {
                                let split: Vec<&str> = name.split(".").collect();
                                let category_name = split[0];
                                let object_name =
                                    match normal_export.base_export.class_index.is_import() {
                                        true => asset
                                            .get_import(normal_export.base_export.class_index)
                                            .map(|e| e.object_name.content.clone())
                                            .ok_or(io::Error::new(
                                                ErrorKind::Other,
                                                "No such import",
                                            ))?,
                                        false => String::new(),
                                    };
                                if object_name != category_name {
                                    continue;
                                }
                                String::from(split[1])
                            }
                            false => name.clone(),
                        };

                        if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                            if array_property.name.content == arr_name {
                                item_types_property
                                    .entry(name.clone())
                                    .or_insert_with(|| Vec::new())
                                    .push((
                                        i,
                                        array_property
                                            .array_type
                                            .as_ref()
                                            .map(|e| e.content.clone())
                                            .ok_or(io::Error::new(
                                                ErrorKind::Other,
                                                "Unknown array type",
                                            ))?,
                                        name.clone(),
                                    ));
                            }
                        }
                    }
                }
            }
        }

        for (name, paths) in entries {
            if !item_types_property.contains_key(name) {
                continue;
            }

            for item_path in paths {
                let (real_name, class_name, soft_class_name) = match item_path.contains(".") {
                    true => {
                        let split: Vec<&str> = item_path.split(".").collect();
                        (
                            String::from(split[0]),
                            String::from(split[1]),
                            String::from(split[1]),
                        )
                    }
                    false => (
                        item_path.clone(),
                        Path::new(item_path)
                            .file_stem()
                            .map(|e| e.to_str())
                            .flatten()
                            .map(|e| String::from(e) + "_C")
                            .ok_or(io::Error::new(ErrorKind::Other, "Invalid item_path"))?,
                        Path::new(item_path)
                            .file_stem()
                            .map(|e| e.to_str())
                            .flatten()
                            .map(|e| String::from(e))
                            .ok_or(io::Error::new(ErrorKind::Other, "Invalid item_path"))?,
                    ),
                };

                let mut blueprint_generated_class_import = None;

                for (export_index, array_type, name) in item_types_property.get(item_path).unwrap()
                {
                    match array_type.as_str() {
                        "ObjectProperty" => {
                            if blueprint_generated_class_import.is_none() {
                                asset.add_fname(&real_name);
                                asset.add_fname(&class_name);

                                let package_import = Import {
                                    class_package: FName::from_slice("/Script/CoreUObject"),
                                    class_name: FName::from_slice("Package"),
                                    outer_index: PackageIndex::new(0),
                                    object_name: FName::from_slice(&real_name),
                                };
                                let package_import = asset.add_import(package_import);

                                let new_import = Import {
                                    class_package: FName::from_slice("/Script/Engine"),
                                    class_name: FName::from_slice("BlueprintGeneratedClass"),
                                    outer_index: package_import,
                                    object_name: FName::from_slice(&class_name),
                                };
                                blueprint_generated_class_import =
                                    Some(asset.add_import(new_import));
                            }

                            let export = asset
                                .exports
                                .get_mut(*export_index)
                                .map(|e| cast!(Export, NormalExport, e))
                                .flatten()
                                .expect("Corrupted memory");
                            export.properties.push(
                                ObjectProperty {
                                    name: FName::from_slice(name),
                                    property_guid: None,
                                    duplication_index: 0,
                                    value: blueprint_generated_class_import.unwrap(),
                                }
                                .into(),
                            );
                        }
                        "SoftObjectProperty" => {
                            asset.add_fname(&real_name);
                            asset.add_name_reference(
                                real_name.clone() + "." + &soft_class_name,
                                false,
                            );

                            let export = asset
                                .exports
                                .get_mut(*export_index)
                                .map(|e| cast!(Export, NormalExport, e))
                                .flatten()
                                .expect("Corrupted memory");

                            export.properties.push(
                                SoftObjectProperty {
                                    name: FName::from_slice(name),
                                    property_guid: None,
                                    duplication_index: 0,
                                    value: FName::new(
                                        real_name.clone() + "." + &soft_class_name,
                                        0,
                                    ),
                                    id: 0,
                                }
                                .into(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        write_asset(integrated_pak, &asset, &name)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }
    Ok(())
}
