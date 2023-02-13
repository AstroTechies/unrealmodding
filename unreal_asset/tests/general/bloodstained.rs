use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[allow(clippy::duplicate_mod)]
#[path = "../shared.rs"]
mod shared;

const TEST_ASSETS: [&[u8]; 6] = [
    include_bytes!("../assets/general/Bloodstained/m01SIP_000_BG.umap"),
    include_bytes!("../assets/general/Bloodstained/m01SIP_000_Gimmick.umap"),
    include_bytes!("../assets/general/Bloodstained/m02VIL_004_Gimmick.umap"),
    include_bytes!("../assets/general/Bloodstained/m05SAN_000_Gimmick.umap"),
    include_bytes!("../assets/general/Bloodstained/PB_DT_ItemMaster.uasset"),
    include_bytes!("../assets/general/Bloodstained/PB_DT_RandomizerRoomCheck.uasset"),
];

#[test]
fn bloodstained() -> Result<(), Error> {
    for test_asset in TEST_ASSETS {
        let mut asset = Asset::new(Cursor::new(test_asset.to_vec()), None);
        asset.set_engine_version(EngineVersion::VER_UE4_18);

        asset.parse_data()?;
        shared::verify_binary_equality(test_asset, None, &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }
    Ok(())
}
