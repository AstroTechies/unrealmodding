use unreal_asset::{
    cast,
    engine_version::EngineVersion,
    error::Error,
    exports::{normal_export::NormalExport, Export, ExportBaseTrait},
    flags::EObjectFlags,
    properties::{object_property::ObjectProperty, PropertyDataTrait},
    types::{FName, PackageIndex},
    Asset,
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
    let mut asset = Asset::new(TEST_ASSET.to_vec(), None);
    asset.set_engine_version(EngineVersion::VER_UE4_23);

    asset.parse_data()?;
    shared::verify_binary_equality(TEST_ASSET, None, &mut asset)?;

    let cdo_export: &mut NormalExport = asset
        .exports
        .iter_mut()
        .find(|e| {
            e.get_base_export().object_flags & EObjectFlags::RF_CLASS_DEFAULT_OBJECT.bits()
                == EObjectFlags::RF_CLASS_DEFAULT_OBJECT.bits()
        })
        .map(|e| cast!(Export, NormalExport, e))
        .flatten()
        .expect("Failed to find cdo export");

    let pickup_actor = cdo_export
        .properties
        .iter_mut()
        .find(|e| e.get_name().content == "PickupActor")
        .expect("Failed to find PickupActor");

    *pickup_actor = ObjectProperty {
        name: FName::from_slice("PickupActor"),
        property_guid: None,
        duplication_index: 0,
        value: PackageIndex::new(0),
    }
    .into();

    Ok(())
}
