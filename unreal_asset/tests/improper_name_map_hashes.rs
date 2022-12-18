mod shared;

use std::collections::HashMap;

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/improper_name_map_hashes/"
        )
    };
}

const ASSET_FILE: &[u8] = include_bytes!(concat!(assets_folder!(), "OC_Gatling_DamageB_B.uasset"));
const ASSET_BULK_FILE: &[u8] =
    include_bytes!(concat!(assets_folder!(), "OC_Gatling_DamageB_B.uexp"));

#[test]
fn improper_name_map_hashes() -> Result<(), Error> {
    let mut asset = Asset::new(ASSET_FILE.to_vec(), Some(ASSET_BULK_FILE.to_vec()));
    asset.set_engine_version(EngineVersion::VER_UE4_25);

    asset.parse_data()?;
    shared::verify_reparse(&mut asset)?;

    let mut testing_entries = HashMap::from([
        ("/Game/WeaponsNTools/GatlingGun/Overclocks/OC_BonusesAndPenalties/OC_Bonus_MovmentBonus_150p".to_string(), false),
        ("/Game/WeaponsNTools/GatlingGun/Overclocks/OC_BonusesAndPenalties/OC_Bonus_MovmentBonus_150p.OC_Bonus_MovmentBonus_150p".to_string(), false)
    ]);

    for (name, hash) in &asset.override_name_map_hashes {
        if let Some(entry) = testing_entries.get_mut(name) {
            assert_eq!(*hash, 0);
            *entry = true;
        }
    }

    for (_, flag) in testing_entries {
        assert!(flag);
    }

    Ok(())
}
