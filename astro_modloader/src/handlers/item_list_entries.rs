use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::{Export, ExportNormalTrait},
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

#[allow(clippy::ptr_arg)]
pub(crate) fn handle_item_list_entries(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    _mod_paks: &mut Vec<PakFile>,
    item_list_entires_maps: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    let mut new_items = HashMap::new();

    for item_list_entries_map in &item_list_entires_maps {
        let item_list_entries_map = item_list_entries_map
            .as_object()
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid item_list_entries"))?;

        for (name, item_list_entries) in item_list_entries_map {
            let item_list_entries = item_list_entries
                .as_object()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid item_list_entries"))?;
            let new_items_entry = new_items.entry(name.clone()).or_insert_with(HashMap::new);

            for (item_name, entries) in item_list_entries {
                let entries = entries
                    .as_array()
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid item_list_entries"))?;

                let new_items_entry_map = new_items_entry
                    .entry(item_name.clone())
                    .or_insert_with(Vec::new);
                for entry in entries {
                    let entry = entry.as_str().ok_or_else(|| {
                        io::Error::new(ErrorKind::Other, "Invalid item_list_entries")
                    })?;
                    new_items_entry_map.push(String::from(entry));
                }
            }
        }
    }

    for (asset_name, entries) in &new_items {
        let asset_name = game_to_absolute(asset_name)
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid asset name"))?;
        let mut asset = get_asset(integrated_pak, game_paks, &asset_name, VER_UE4_23)?;

        let mut item_types_property: HashMap<String, Vec<(usize, usize, String)>> = HashMap::new();
        for i in 0..asset.exports.len() {
            if let Some(normal_export) = asset.exports[i].get_normal_export() {
                for j in 0..normal_export.properties.len() {
                    let property = &normal_export.properties[j];
                    for entry_name in entries.keys() {
                        let mut arr_name = entry_name.clone();
                        if arr_name.contains('.') {
                            let split: Vec<&str> = arr_name.split('.').collect();
                            let export_name = split[0].to_owned();
                            arr_name = split[1].to_owned();

                            if normal_export.base_export.class_index.is_import() {
                                if asset
                                    .get_import(normal_export.base_export.class_index)
                                    .map(|e| e.object_name.content != export_name)
                                    .unwrap_or(true)
                                {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                            if array_property.name.content == arr_name {
                                item_types_property
                                    .entry(entry_name.clone())
                                    .or_insert_with(Vec::new)
                                    .push((
                                        i,
                                        j,
                                        array_property
                                            .array_type
                                            .as_ref()
                                            .ok_or_else(|| {
                                                io::Error::new(
                                                    ErrorKind::Other,
                                                    "Invalid array_property",
                                                )
                                            })?
                                            .content
                                            .clone(),
                                    ));
                            }
                        }
                    }
                }
            }
        }
        for (name, item_paths) in entries {
            if !item_types_property.contains_key(name) {
                continue;
            }
            for item_path in item_paths {
                let (real_name, class_name, soft_class_name) = match item_path.contains('.') {
                    true => {
                        let split: Vec<&str> = item_path.split('.').collect();
                        (
                            split[0].to_string(),
                            split[1].to_string(),
                            split[1].to_string(),
                        )
                    }
                    false => (
                        item_path.clone(),
                        Path::new(item_path)
                            .file_stem()
                            .and_then(|e| e.to_str())
                            .map(|e| String::from(e) + "_C")
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid item_path"))?,
                        Path::new(item_path)
                            .file_stem()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_string())
                            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid item_path"))?,
                    ),
                };

                let mut new_import = PackageIndex::new(0);

                for (export_index, property_index, array_type) in
                    item_types_property.get(name).unwrap()
                {
                    match array_type.as_str() {
                        "ObjectProperty" => {
                            if new_import.index == 0 {
                                asset.add_name_reference(real_name.clone(), false);
                                asset.add_name_reference(class_name.clone(), false);

                                let inner_import = Import {
                                    class_package: FName::from_slice("/Script/CoreUObject"),
                                    class_name: FName::from_slice("Package"),
                                    outer_index: PackageIndex::new(0),
                                    object_name: FName::new(real_name.clone(), 0),
                                };
                                let inner_import = asset.add_import(inner_import);

                                let import = Import {
                                    class_package: FName::from_slice("/Script/Engine"),
                                    class_name: FName::from_slice("BlueprintGeneratedClass"),
                                    outer_index: inner_import,
                                    object_name: FName::new(class_name.clone(), 0),
                                };
                                new_import = asset.add_import(import);
                            }

                            let export =
                                cast!(Export, NormalExport, &mut asset.exports[*export_index])
                                    .expect("Corrupted memory");
                            let property = cast!(
                                Property,
                                ArrayProperty,
                                &mut export.properties[*property_index]
                            )
                            .expect("Corrupted memory");
                            property.value.push(
                                ObjectProperty {
                                    name: property.name.clone(),
                                    property_guid: None,
                                    duplication_index: 0,
                                    value: new_import,
                                }
                                .into(),
                            );
                        }
                        "SoftObjectProperty" => {
                            asset.add_name_reference(real_name.clone(), false);
                            asset.add_name_reference(
                                real_name.clone() + "." + &soft_class_name,
                                false,
                            );

                            let export =
                                cast!(Export, NormalExport, &mut asset.exports[*export_index])
                                    .expect("Corrupted memory");
                            let property = cast!(
                                Property,
                                ArrayProperty,
                                &mut export.properties[*property_index]
                            )
                            .expect("Corrupted memory");
                            property.value.push(
                                SoftObjectProperty {
                                    name: property.name.clone(),
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

        write_asset(integrated_pak, &asset, &asset_name)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }

    Ok(())
}
