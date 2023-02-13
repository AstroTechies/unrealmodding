use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[path = "../shared.rs"]
mod shared;

const TEST_ASSETS: [(&[u8], &[u8]); 2] = [
    (
        include_bytes!("../assets/general/Misc_426/MainChar_BellySlice_BR.uasset"),
        include_bytes!("../assets/general/Misc_426/MainChar_BellySlice_BR.uexp"),
    ),
    (
        include_bytes!("../assets/general/Misc_426/RaceSimDataAsset.uasset"),
        include_bytes!("../assets/general/Misc_426/RaceSimDataAsset.uexp"),
    ),
];

#[test]
fn misc426() -> Result<(), Error> {
    for (test_asset, asset_bulk) in TEST_ASSETS {
        let mut asset = Asset::new(
            Cursor::new(test_asset.to_vec()),
            Some(Cursor::new(asset_bulk.to_vec())),
        );
        asset.set_engine_version(EngineVersion::VER_UE4_26);

        asset.parse_data()?;
        shared::verify_binary_equality(test_asset, Some(asset_bulk), &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }

    Ok(())
}
