use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

#[path = "../shared.rs"]
mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/StarlitSeason/"
        )
    };
}

const TEST_ASSETS: [(&[u8], &[u8]); 1] = [(
    include_bytes!(concat!(
        assets_folder!(),
        "CharacterCostume_chr0001_DataTable.uasset"
    )),
    include_bytes!(concat!(
        assets_folder!(),
        "CharacterCostume_chr0001_DataTable.uexp"
    )),
)];

#[test]
fn starlit_season() -> Result<(), Error> {
    for (test_asset, asset_bulk) in TEST_ASSETS {
        let mut asset = Asset::new(test_asset.to_vec(), Some(asset_bulk.to_vec()));
        asset.set_engine_version(EngineVersion::VER_UE4_24);

        asset.parse_data()?;
        shared::verify_binary_equality(test_asset, Some(asset_bulk), &mut asset)?;
        assert!(shared::verify_all_exports_parsed(&asset));
    }
    Ok(())
}
