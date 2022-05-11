use std::{
    io::{self, ErrorKind},
    path::Path,
};

use unreal_asset::{
    exports::Export,
    properties::{object_property::ObjectProperty, Property},
    ue4version::VER_UE4_23,
    unreal_types::{FName, PackageIndex},
    Import,
};
use unreal_modintegrator::write_asset;
use unreal_pak::PakFile;

use super::{get_asset, MAP_PATHS};

pub(crate) fn handle_mission_trailheads(
    _data: &(),
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    trailhead_arrays: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    for map_path in MAP_PATHS {
        let mut asset = get_asset(
            integrated_pak,
            game_paks,
            &String::from(map_path),
            VER_UE4_23,
        )?;

        let mut additional_properties: Vec<Property> = Vec::new();

        for trailheads in &trailhead_arrays {
            let trailheads = trailheads
                .as_array()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailheads"))?;
            for trailhead in trailheads {
                asset.add_name_reference(String::from("AstroMissionDataAsset"), false);

                let trailhead = trailhead
                    .as_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailheads"))?;
                let soft_class_name = Path::new(trailhead)
                    .file_stem()
                    .and_then(|e| e.to_str())
                    .ok_or_else(|| io::Error::new(ErrorKind::Other, "Invalid trailheads"))?;

                asset.add_name_reference(String::from(trailhead), false);
                asset.add_name_reference(String::from(soft_class_name), false);

                let package_import = Import {
                    class_package: FName::from_slice("/Script/CoreUObject"),
                    class_name: FName::from_slice("Package"),
                    outer_index: PackageIndex::new(0),
                    object_name: FName::from_slice(trailhead),
                };
                let package_import = asset.add_import(package_import);

                let new_import = Import {
                    class_package: FName::from_slice("/Script/Astro"),
                    class_name: FName::from_slice("AstroMissionDataAsset"),
                    outer_index: package_import,
                    object_name: FName::from_slice(soft_class_name),
                };
                let new_import = asset.add_import(new_import);

                additional_properties.push(
                    ObjectProperty {
                        name: FName::from_slice("dummy"),
                        property_guid: None,
                        duplication_index: 0,
                        value: new_import,
                    }
                    .into(),
                );
            }
        }

        let mut export_index = None;

        for i in 0..asset.exports.len() {
            if let Export::NormalExport(e) = &asset.exports[i] {
                if e.base_export.class_index.is_import()
                    && asset
                        .get_import(e.base_export.class_index)
                        .map(|e| &e.object_name.content == "AstroSettings")
                        .unwrap_or(false)
                {
                    export_index = Some(i);
                    break;
                }
            }
        }

        if let Some(export_index) = export_index {
            let export = &mut asset.exports[export_index];
            if let Export::NormalExport(export) = export {
                additional_properties.iter_mut().for_each(|e| match e {
                    Property::ObjectProperty(e) => e.name = export.base_export.object_name.clone(),
                    _ => panic!("Corrupted memory"),
                });
                export.properties.extend(additional_properties);
            }
        }

        write_asset(integrated_pak, &asset, &String::from(map_path))
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
    }

    Ok(())
}
