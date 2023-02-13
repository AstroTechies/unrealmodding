use std::io::{self, ErrorKind};
use std::path::Path;

use serde_json::Map;

use unreal_asset::{
    cast,
    exports::{Export, ExportNormalTrait},
    properties::{
        int_property::{BoolProperty, IntProperty},
        object_property::ObjectProperty,
        Property, PropertyDataTrait,
    },
    unreal_types::{FName, PackageIndex},
    Asset, Import,
};
use unreal_pak::PakFile;

use crate::error::Error;
use crate::write_asset;

pub(crate) struct GameInfo {
    pub(crate) game_name: String,
    pub(crate) ue_version: i32,
}

fn create_property(
    asset: &mut Asset,
    property_name: &str,
    property_type: &str,
    data: &serde_json::Value,
) -> Result<Property, io::Error> {
    match property_type {
        "ObjectProperty" => {
            let data = data.as_object().ok_or_else(|| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Invalid data for ObjectProperty {}", data),
                )
            })?;

            let path = data
                .get("Name")
                .map(|e| e.as_str())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, "No name in objectproperty data")
                })?;

            let import_type = data
                .get("ImportType")
                .map(|e| e.as_str())
                .flatten()
                .unwrap_or("BlueprintGeneratedClass");

            let object_name = Path::new(path)
                .file_stem()
                .and_then(|e| e.to_str())
                .map(|e| String::from(e))
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::Other, format!("Invalid path {}", path))
                })?;

            let object_name = match import_type {
                "BlueprintGeneratedClass" => object_name + "_C",
                _ => object_name,
            };

            asset.add_fname("/Script/CoreUObject");
            asset.add_fname("Package");
            asset.add_fname(path);
            asset.add_fname(import_type);
            asset.add_fname(&object_name);
            asset.add_fname(property_name);

            let package_import = asset.find_import_no_index(
                &FName::from_slice("/Script/CoreUObject"),
                &FName::from_slice("Package"),
                &FName::from_slice(path),
            );

            let package_import = match package_import {
                Some(e) => PackageIndex::new(e),
                None => {
                    let import = Import {
                        class_package: FName::from_slice("/Script/CoreUObject"),
                        class_name: FName::from_slice("Package"),
                        outer_index: PackageIndex::new(0),
                        object_name: FName::from_slice(path),
                    };
                    let index = asset.add_import(import);
                    index
                }
            };

            let blueprint_generated_class_import = asset.find_import_no_index(
                &FName::from_slice("/Script/Engine"),
                &FName::from_slice(import_type),
                &FName::from_slice(&object_name),
            );

            let blueprint_generated_class_import = match blueprint_generated_class_import {
                Some(e) => PackageIndex::new(e),
                None => {
                    let import = Import {
                        class_package: FName::from_slice("/Script/Engine"),
                        class_name: FName::from_slice(import_type),
                        outer_index: package_import,
                        object_name: FName::from_slice(&object_name),
                    };
                    let index = asset.add_import(import);
                    index
                }
            };

            let property = ObjectProperty {
                name: FName::from_slice(property_name),
                property_guid: Some([0u8; 16]),
                duplication_index: 0,
                value: blueprint_generated_class_import,
            };

            Ok(property.into())
        }
        "IntProperty" => {
            let value = data.as_i64().ok_or_else(|| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Invalid value for IntProperty {}", data),
                )
            })?;

            let property = IntProperty {
                name: FName::from_slice(property_name),
                property_guid: Some([0u8; 16]),
                duplication_index: 0,
                value: value as i32,
            };

            Ok(property.into())
        }
        "BoolProperty" => {
            let value = data.as_bool().ok_or_else(|| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Invalid value for BoolProperty {}", data),
                )
            })?;

            let property = BoolProperty {
                name: FName::from_slice(property_name),
                property_guid: Some([0u8; 16]),
                duplication_index: 0,
                value,
            };

            Ok(property.into())
        }
        _ => Err(io::Error::new(
            ErrorKind::Other,
            format!("Property type {} not supported", property_type),
        )),
    }
}

fn handle_property(
    asset: &mut Asset,
    property_name: &String,
    export_index: usize,
    property_index: usize,
    property_info: &Map<String, serde_json::Value>,
) -> Result<(), io::Error> {
    let property_type = property_info
        .get("Type")
        .map(|e| e.as_str())
        .flatten()
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::Other,
                format!("No property type specified for property {}", property_name),
            )
        })?;

    match property_type {
        "MapProperty" => {
            let key_type = property_info
                .get("KeyType")
                .map(|e| e.as_str())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("No key type on property {}", property_name),
                    )
                })?;

            let value_type = property_info
                .get("ValueType")
                .map(|e| e.as_str())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("No value type on property {}", property_name),
                    )
                })?;

            let data = property_info
                .get("Data")
                .map(|e| e.as_array())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("Invalid data for property {}", property_name),
                    )
                })?;

            for modified_property in data {
                let modified_property = modified_property.as_object().ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("Invalid data for property {}", property_name),
                    )
                })?;

                let key_data = modified_property.get("Key").ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("No key on data for property {}", property_name),
                    )
                })?;

                let value_data = modified_property.get("Value").ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("No value on data for property {}", property_name),
                    )
                })?;

                let key = create_property(asset, "Key", key_type, key_data)?;
                let value = create_property(asset, "Value", value_type, value_data)?;

                let export = (&mut asset.exports[export_index])
                    .get_normal_export_mut()
                    .unwrap();

                let property = cast!(
                    Property,
                    MapProperty,
                    &mut export.properties[property_index]
                )
                .unwrap();
                property.value.insert(key, value);
            }
            Ok(())
        }
        "ArrayProperty" => Ok(()),
        _ => Err(io::Error::new(
            ErrorKind::Other,
            format!("Property type {} is not supported", property_type),
        )),
    }
}

#[allow(clippy::ptr_arg)]
pub(crate) fn integrate(
    game_info: &GameInfo,
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    _mod_paks: &mut Vec<PakFile>,
    custom: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    for custom_mod in custom {
        let entries = custom_mod
            .as_object()
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid asset entry"))?;

        for (asset_name, modifications) in entries {
            // todo: open asset
            let asset_name =
                unreal_modmetadata::game_to_absolute(&game_info.game_name, asset_name.as_str())
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid asset path"))?;

            let mut asset =
                get_asset(integrated_pak, game_paks, &asset_name, game_info.ue_version)?;

            let property_modifications = modifications.as_object().ok_or_else(|| {
                io::Error::new(ErrorKind::Other, "Invalid property modifications")
            })?;

            for (property_name, modification) in property_modifications {
                // todo: if this lags, optimize
                let mut export_index = None;

                let (export_name, property_name) = match property_name.contains(".") {
                    true => {
                        let split: Vec<&str> = property_name.split(".").collect();
                        (String::from(split[0]), Some(String::from(split[1])))
                    }
                    false => (property_name.clone(), None),
                };

                for i in 0..asset.exports.len() {
                    let export = &asset.exports[i];
                    if let Some(normal_export) = export.get_normal_export() {
                        if normal_export.base_export.object_name.content == export_name {
                            export_index = Some(i);
                            break;
                        }
                    }
                }

                let export_index = export_index
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such property"))?;

                let export = (&asset.exports[export_index]).get_normal_export().unwrap();

                if let Some(property_name) = property_name {
                    let mut property_index = None;
                    for i in 0..export.properties.len() {
                        if &export.properties[i].get_name().content == &property_name {
                            property_index = Some(i);
                            break;
                        }
                    }

                    let property_index = property_index.ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("No such property {}", property_name),
                        )
                    })?;

                    let property_info = modification.as_object().ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("Invalid property modification for {}", property_name),
                        )
                    })?;

                    handle_property(
                        &mut asset,
                        &property_name,
                        export_index,
                        property_index,
                        property_info,
                    )?;
                }
            }

            write_asset(integrated_pak, &asset, &asset_name)
                .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        }
    }

    Ok(())
}
