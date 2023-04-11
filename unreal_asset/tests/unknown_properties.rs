use std::{collections::HashMap, io::Cursor};

use unreal_asset::{
    cast, engine_version::EngineVersion, error::Error, exports::ExportNormalTrait,
    properties::Property, Asset,
};

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/unknown_properties/"
        )
    };
}

const TEST_ASSET: &[u8] = include_bytes!(concat!(assets_folder!(), "BP_DetPack_Charge.uasset"));
const TEST_BULK: &[u8] = include_bytes!(concat!(assets_folder!(), "BP_DetPack_Charge.uexp"));

#[test]
fn unknown_properties() -> Result<(), Error> {
    let mut asset = Asset::new(Cursor::new(TEST_ASSET), Some(Cursor::new(TEST_BULK)));
    asset.set_engine_version(EngineVersion::VER_UE4_25);

    asset.parse_data()?;
    shared::verify_binary_equality(TEST_ASSET, Some(TEST_BULK), &mut asset)?;
    assert!(shared::verify_all_exports_parsed(&asset));

    let mut new_unknown_properties = HashMap::from([
        ("GarbagePropty", false),
        ("EvenMoreGarbageTestingPropertyy", false),
    ]);

    for export in &asset.exports {
        if let Some(normal_export) = export.get_normal_export() {
            for property in &normal_export.properties {
                if let Some(unknown_property) = cast!(Property, UnknownProperty, property) {
                    if let Some(entry) = new_unknown_properties
                        .get_mut(unknown_property.serialized_type.content.as_str())
                    {
                        *entry = true;
                    } else {
                        panic!("Test failed!");
                    }
                }
            }
        }
    }

    for (_, flag) in new_unknown_properties {
        assert!(flag);
    }

    Ok(())
}
