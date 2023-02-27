use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/Versioned/"
        )
    };
}

const TEST_ASSETS: [&[u8]; 1] = [include_bytes!(concat!(
    assets_folder!(),
    "Assault_M1A1Thompson_WW2_DrumSuppressor.uasset"
))];

#[test]
fn versioned() -> Result<(), Error> {
    for test_asset in TEST_ASSETS {
        let mut asset = Asset::new(Cursor::new(test_asset.to_vec()), None);
        asset.set_engine_version(EngineVersion::UNKNOWN);

        asset.parse_data()?;
        shared::verify_binary_equality(test_asset, None, &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }
    Ok(())
}
