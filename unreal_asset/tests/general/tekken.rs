use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, Asset, Error};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

macro_rules! assets_folder {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/general/Tekken/")
    };
}

const TEST_ASSETS: [&[u8]; 1] = [include_bytes!(concat!(
    assets_folder!(),
    "BP_TekkenPlayer_Modular.uasset"
))];

#[test]
fn tekken() -> Result<(), Error> {
    for test_asset in TEST_ASSETS {
        let mut asset = Asset::new(
            Cursor::new(test_asset),
            None,
            EngineVersion::VER_UE4_14,
            None,
        )?;
        shared::verify_binary_equality(test_asset, None, &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }

    Ok(())
}
