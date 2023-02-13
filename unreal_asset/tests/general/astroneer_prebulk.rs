use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

const TEST_ASSETS: [&[u8]; 5] = [
    include_bytes!("../assets/general/Astroneer_prebulk/Augment_BroadBrush.uasset"),
    include_bytes!("../assets/general/Astroneer_prebulk/DebugMenu.uasset"),
    include_bytes!("../assets/general/Astroneer_prebulk/LargeResourceCanister_IT.uasset"),
    include_bytes!("../assets/general/Astroneer_prebulk/ResourceProgressCurve.uasset"),
    include_bytes!("../assets/general/Astroneer_prebulk/Staging_T2.umap"),
];

#[test]
fn astroneer_prebulk() -> Result<(), Error> {
    for test_asset in TEST_ASSETS {
        let mut asset = Asset::new(Cursor::new(test_asset.to_vec()), None);
        asset.set_engine_version(EngineVersion::VER_UE4_23);
        asset.parse_data()?;
        shared::verify_binary_equality(test_asset, None, &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }

    Ok(())
}
