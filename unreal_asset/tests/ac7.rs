use std::io::Cursor;

use unreal_asset::{ac7::*, engine_version::EngineVersion, error::Error, Asset};

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/ac7/")
    };
}

const TEST_ASSETS: [(&str, &[u8], &[u8]); 2] = [
    (
        "ex02_IGC_03_Subtitle",
        include_bytes!(concat!(assets_folder!(), "ex02_IGC_03_Subtitle.uasset")),
        include_bytes!(concat!(assets_folder!(), "ex02_IGC_03_Subtitle.uexp")),
    ),
    (
        "plwp_6aam_a0",
        include_bytes!(concat!(assets_folder!(), "plwp_6aam_a0.uasset")),
        include_bytes!(concat!(assets_folder!(), "plwp_6aam_a0.uexp")),
    ),
];

#[test]
fn ac7() -> Result<(), Error> {
    for (name, asset_data, bulk_data) in TEST_ASSETS {
        let key = AC7XorKey::new(name);
        let mut parsed = Asset::new(
            AC7Reader::new(key, Cursor::new(asset_data)),
            Some(AC7Reader::new(key, Cursor::new(bulk_data))),
            EngineVersion::VER_UE4_18,
            None,
        )?;

        shared::verify_binary_equality(&asset_data, Some(&bulk_data), &mut parsed)?;
        shared::verify_all_exports_parsed(&parsed);

        let mut data = AC7Writer::new(key, Cursor::new(Vec::new()));
        let mut bulk = AC7Writer::new(key, Cursor::new(Vec::new()));
        parsed.write_data(&mut data, Some(&mut bulk))?;

        let data = data.into_inner().into_inner();
        let bulk = bulk.into_inner().into_inner();

        assert_eq!(asset_data, data);
        assert_eq!(bulk_data, bulk);
    }

    Ok(())
}
