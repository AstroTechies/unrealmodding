mod shared;

use std::io::Cursor;

use unreal_asset::{
    cast,
    engine_version::EngineVersion,
    error::Error,
    exports::{normal_export::NormalExport, ExportNormalTrait},
    properties::{
        map_property::MapProperty, struct_property::StructProperty, Property, PropertyDataTrait,
    },
    types::PackageIndex,
    Asset,
};

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/custom_serialization_structs_in_map/"
        )
    };
}

const ASSET_FILE: &[u8] = include_bytes!(concat!(assets_folder!(), "asset.uasset"));
const ASSET_BULK_FILE: &[u8] = include_bytes!(concat!(assets_folder!(), "asset.uexp"));

#[test]
fn custom_serialization_structs_in_map() -> Result<(), Error> {
    let mut asset = Asset::new(
        Cursor::new(ASSET_FILE),
        Some(Cursor::new(ASSET_BULK_FILE)),
        EngineVersion::VER_UE4_25,
    )?;

    shared::verify_binary_equality(ASSET_FILE, Some(ASSET_BULK_FILE), &mut asset)?;

    let export_two = asset
        .get_export(PackageIndex::new(2))
        .ok_or_else(|| Error::no_data("Export two doesn't exist".to_string()))?;
    let export_two: &NormalExport = export_two
        .get_normal_export()
        .ok_or_else(|| Error::invalid_file("Export two is not NormalExport".to_string()))?;

    let test_map = export_two
        .properties
        .iter()
        .find(|e| e.get_name().content == "KekWait")
        .ok_or_else(|| {
            Error::invalid_file("Export doesn't contain a KekWait property".to_string())
        })?;
    let test_map: &MapProperty = cast!(Property, MapProperty, test_map)
        .ok_or_else(|| Error::invalid_file("KekWait property is not MapProperty".to_string()))?;

    let (_, entry_key, entry_value) = test_map.value.iter().next().ok_or_else(|| {
        Error::invalid_file("KekWait MapProperty doesn't have any entries".to_string())
    })?;

    let entry_key: &StructProperty =
        cast!(Property, StructProperty, entry_key).ok_or_else(|| {
            Error::invalid_file(
                "KekWait MapProperty's first entry's key is not StructProperty".to_string(),
            )
        })?;
    let entry_value: &StructProperty =
        cast!(Property, StructProperty, entry_value).ok_or_else(|| {
            Error::invalid_file(
                "KekWait MapProperty's first entry's value is not StructProperty".to_string(),
            )
        })?;

    let first_key_value = entry_key.value.first().ok_or_else(|| {
        Error::invalid_file(
            "KekWait MapProperty's first entry's key doesn't contain a value".to_string(),
        )
    })?;

    let first_value_value = entry_value.value.first().ok_or_else(|| {
        Error::invalid_file(
            "KekWait MapProperty's first entry's value doesn't contain a value".to_string(),
        )
    })?;

    assert!(cast!(Property, VectorProperty, first_key_value).is_some());
    assert!(cast!(Property, LinearColorProperty, first_value_value).is_some());

    Ok(())
}
