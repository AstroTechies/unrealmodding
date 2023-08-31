use std::io::Cursor;

use unreal_asset::{
    cast,
    engine_version::EngineVersion,
    exports::{Export, ExportBaseTrait, NormalExport},
    flags::EObjectFlags,
    properties::{object_property::ObjectProperty, PropertyDataTrait},
    types::PackageIndex,
    unversioned::Ancestry,
    Asset, Error,
};

mod shared;

macro_rules! test_asset {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/Astroneer_prebulk/Augment_BroadBrush"
        )
    };
}

const TEST_ASSET: &[u8] = include_bytes!(concat!(test_asset!(), ".uasset"));

#[test]
fn cdo_modification() -> Result<(), Error> {
    let mut asset = Asset::new(
        Cursor::new(TEST_ASSET),
        None,
        EngineVersion::VER_UE4_23,
        None,
    )?;

    shared::verify_binary_equality(TEST_ASSET, None, &mut asset)?;

    let new_name = asset.get_name_map().get_mut().add_fname("PickupActor");

    let cdo_export: &mut NormalExport<_> = asset
        .asset_data
        .exports
        .iter_mut()
        .find(|e| {
            e.get_base_export().object_flags & EObjectFlags::RF_CLASS_DEFAULT_OBJECT
                == EObjectFlags::RF_CLASS_DEFAULT_OBJECT
        })
        .and_then(|e| cast!(Export, NormalExport, e))
        .expect("Failed to find cdo export");

    let pickup_actor = cdo_export
        .properties
        .iter_mut()
        .find(|e| e.get_name() == "PickupActor")
        .expect("Failed to find PickupActor");

    *pickup_actor = ObjectProperty {
        name: new_name,
        property_guid: None,
        duplication_index: 0,
        value: PackageIndex::new(0),
        ancestry: Ancestry::default(),
    }
    .into();

    Ok(())
}
