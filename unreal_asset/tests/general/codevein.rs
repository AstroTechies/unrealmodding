use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/CodeVein/"
        )
    };
}

const TEST_ASSETS: [(&[u8], &[u8]); 1] = [(
    include_bytes!(concat!(assets_folder!(), "SK_Inner_Female1.uasset")),
    include_bytes!(concat!(assets_folder!(), "SK_Inner_Female1.uexp")),
)];

#[test]
fn codevein() -> Result<(), Error> {
    for (test_asset, asset_bulk) in TEST_ASSETS {
        let mut asset = Asset::new(
            Cursor::new(test_asset),
            Some(Cursor::new(asset_bulk)),
            EngineVersion::VER_UE4_18,
            None,
        )?;
        shared::verify_binary_equality(test_asset, Some(asset_bulk), &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }

    Ok(())
}
