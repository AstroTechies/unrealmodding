use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, Asset, Error};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/BloodStained/"
        )
    };
}

const TEST_ASSETS: [&[u8]; 6] = [
    include_bytes!(concat!(assets_folder!(), "m01SIP_000_BG.umap")),
    include_bytes!(concat!(assets_folder!(), "m01SIP_000_Gimmick.umap")),
    include_bytes!(concat!(assets_folder!(), "m02VIL_004_Gimmick.umap")),
    include_bytes!(concat!(assets_folder!(), "m05SAN_000_Gimmick.umap")),
    include_bytes!(concat!(assets_folder!(), "PB_DT_ItemMaster.uasset")),
    include_bytes!(concat!(
        assets_folder!(),
        "PB_DT_RandomizerRoomCheck.uasset"
    )),
];

#[test]
fn bloodstained() -> Result<(), Error> {
    for test_asset in TEST_ASSETS {
        let mut asset = Asset::new(
            Cursor::new(test_asset),
            None,
            EngineVersion::VER_UE4_18,
            None,
        )?;
        shared::verify_binary_equality(test_asset, None, &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }
    Ok(())
}
