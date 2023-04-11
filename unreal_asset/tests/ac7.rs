use std::io::Cursor;

use unreal_asset::{
    ac7::{self, AC7XorKey},
    engine_version::EngineVersion,
    error::Error,
    Asset,
};

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
        let (decrypted_data, decrypted_bulk) = ac7::decrypt(asset_data, bulk_data, key);

        let mut parsed = Asset::new(
            Cursor::new(decrypted_data.as_slice()),
            Some(Cursor::new(decrypted_bulk.as_slice())),
            EngineVersion::VER_UE4_18,
        )?;

        shared::verify_binary_equality(&decrypted_data, Some(&decrypted_bulk), &mut parsed)?;
        shared::verify_all_exports_parsed(&parsed);

        let mut data = Cursor::new(Vec::new());
        let mut bulk = Cursor::new(Vec::new());
        parsed.write_data(&mut data, Some(&mut bulk))?;

        let data = data.into_inner();
        let bulk = bulk.into_inner();

        let key = AC7XorKey::new(name);
        let (encrypted_data, encrypted_bulk) = ac7::encrypt(&data, &bulk, key);

        assert_eq!(asset_data, encrypted_data);
        assert_eq!(bulk_data, encrypted_bulk);
    }

    Ok(())
}
