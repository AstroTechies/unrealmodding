use std::io::Cursor;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/ue5/")
    };
}

const TEST_ASSETS: [(&[u8], &[u8]); 2] = [
    (
        include_bytes!(concat!(assets_folder!(), "PublicHousingPlot_Root.umap")),
        include_bytes!(concat!(assets_folder!(), "PublicHousingPlot_Root.uexp")),
    ),
    (
        include_bytes!(concat!(assets_folder!(), "Village_Root.umap")),
        include_bytes!(concat!(assets_folder!(), "Village_Root.uexp")),
    ),
];

#[test]
fn ue5() -> Result<(), Error> {
    for (asset_data, bulk_data) in TEST_ASSETS {
        let mut parsed = Asset::new(
            Cursor::new(asset_data),
            Some(Cursor::new(bulk_data)),
            EngineVersion::VER_UE5_2,
            None,
        )?;

        shared::verify_binary_equality(asset_data, Some(bulk_data), &mut parsed)?;
        shared::verify_all_exports_parsed(&parsed);
    }

    Ok(())
}
