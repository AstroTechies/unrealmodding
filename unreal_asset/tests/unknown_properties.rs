use std::{collections::HashMap, io::Cursor};

use unreal_asset::{
    cast, engine_version::EngineVersion, error::Error, properties::Property, Asset,
};
use unreal_asset_exports::ExportNormalTrait;

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
    let mut asset = Asset::new(
        Cursor::new(TEST_ASSET),
        Some(Cursor::new(TEST_BULK)),
        EngineVersion::VER_UE4_25,
        None,
    )?;
    shared::verify_binary_equality(TEST_ASSET, Some(TEST_BULK), &mut asset)?;
    assert!(shared::verify_all_exports_parsed(&asset));

    let mut new_unknown_properties = HashMap::from([
        ("GarbagePropty", false),
        ("EvenMoreGarbageTestingPropertyy", false),
    ]);

    for export in &asset.asset_data.exports {
        if let Some(normal_export) = export.get_normal_export() {
            for property in &normal_export.properties {
                if let Some(unknown_property) = cast!(Property, UnknownProperty, property) {
                    if let Some(entry) = unknown_property
                        .serialized_type
                        .get_content(|unk| new_unknown_properties.get_mut(unk))
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
