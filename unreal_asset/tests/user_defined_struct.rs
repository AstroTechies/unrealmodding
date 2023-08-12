use std::io::Cursor;

use unreal_asset::{cast, engine_version::EngineVersion, error::Error, Asset};
use unreal_asset_exports::Export;

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/user_defined_struct/"
        )
    };
}

const TEST_ASSETS: [(&[u8], &[u8]); 1] = [(
    include_bytes!(concat!(
        assets_folder!(),
        "achievements_STRUCT_entry.uasset"
    )),
    include_bytes!(concat!(assets_folder!(), "achievements_STRUCT_entry.uexp")),
)];

#[test]
fn user_defined_struct() -> Result<(), Error> {
    for (asset_data, bulk_data) in TEST_ASSETS {
        let mut parsed = Asset::new(
            Cursor::new(asset_data),
            Some(Cursor::new(bulk_data)),
            EngineVersion::VER_UE4_26,
            None,
        )?;
        shared::verify_binary_equality(asset_data, Some(bulk_data), &mut parsed)?;
        shared::verify_all_exports_parsed(&parsed);
        let uds = parsed
            .asset_data
            .exports
            .iter()
            .find_map(|ex| cast!(Export, UserDefinedStructExport, ex))
            .expect("asset does not contain a user defined struct");
        // run tests with -- --nocapture to see output
        dbg!(uds);
    }

    Ok(())
}
