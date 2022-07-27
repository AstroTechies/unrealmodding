use std::{
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    cast,
    exports::{Export, ExportNormalTrait},
    properties::{object_property::ObjectProperty, Property},
    reader::asset_trait::AssetTrait,
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Import,
};
use unreal_modintegrator::{helpers::get_asset, write_asset};
use unreal_pak::PakFile;

use super::MAP_PATHS;

#[allow(clippy::ptr_arg)]
pub(crate) fn handle_mission_trailheads(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    _mod_paks: &mut Vec<PakFile>,
    trailhead_arrays: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    for map_path in MAP_PATHS {
        let mut asset = get_asset(
            integrated_pak,
            game_paks,
            &String::from(map_path),
            VER_UE4_23,
        )?;

        let mut trailheads = Vec::new();
        for trailheads_array in &trailhead_arrays {
            let trailheads_array = trailheads_array
                .as_array()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailheads"))?;
            for trailhead in trailheads_array {
                let trailhead = trailhead
                    .as_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailheads"))?;
                trailheads.push(trailhead);
            }
        }

        let mut mission_data_export_index = None;
        let mut mission_data_property_index = None;

        for i in 0..asset.exports.len() {
            let export = &asset.exports[i];
            if let Some(normal_export) = export.get_normal_export() {
                if normal_export.base_export.class_index.is_import() {
                    let import = asset
                        .get_import(normal_export.base_export.class_index)
                        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid import"))?;
                    if import.object_name.content == "AstroSettings" {
                        for j in 0..normal_export.properties.len() {
                            let property = &normal_export.properties[j];
                            if let Some(array_property) = cast!(Property, ArrayProperty, property) {
                                if array_property.name.content == "MissionData"
                                    && array_property
                                        .array_type
                                        .as_ref()
                                        .map(|e| e.content == "ObjectProperty")
                                        .unwrap_or(false)
                                {
                                    mission_data_export_index = Some(i);
                                    mission_data_property_index = Some(j);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if let (Some(mission_data_export_index), Some(mission_data_property_index)) =
            (mission_data_export_index, mission_data_property_index)
        {
            for trailhead in trailheads {
                let soft_class_name = Path::new(trailhead)
                    .file_stem()
                    .and_then(|e| e.to_str())
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailhead"))?;
                asset.add_fname(trailhead);
                asset.add_fname(soft_class_name);

                let package_link = Import {
                    class_package: FName::from_slice("/Script/CoreUObject"),
                    class_name: FName::from_slice("Package"),
                    outer_index: PackageIndex::new(0),
                    object_name: FName::from_slice(trailhead),
                };
                let package_link = asset.add_import(package_link);

                let mission_data_asset_link = Import {
                    class_package: FName::from_slice("/Script/Astro"),
                    class_name: FName::from_slice("AstroMissionDataAsset"),
                    outer_index: package_link,
                    object_name: FName::from_slice(soft_class_name),
                };
                let mission_data_asset_link = asset.add_import(mission_data_asset_link);

                let mission_data_export = cast!(
                    Export,
                    NormalExport,
                    &mut asset.exports[mission_data_export_index as usize]
                )
                .expect("Corrupted memory");
                let mission_data_property = cast!(
                    Property,
                    ArrayProperty,
                    &mut mission_data_export.properties[mission_data_property_index as usize]
                )
                .expect("Corrupted memory");

                let property = ObjectProperty {
                    name: mission_data_property.name.clone(),
                    property_guid: Some([0u8; 16]),
                    duplication_index: 0,
                    value: mission_data_asset_link,
                };
                mission_data_property.value.push(property.into());
            }
        }

        write_asset(integrated_pak, &asset, &String::from(map_path))
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }

    Ok(())
}
