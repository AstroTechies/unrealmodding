use std::{collections::HashMap, io::Cursor};

use unreal_asset::{
    cast,
    engine_version::EngineVersion,
    error::Error,
    exports::{data_table_export::DataTableExport, Export},
    properties::{Property, PropertyDataTrait},
    Asset,
};

mod shared;

macro_rules! test_asset {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/general/BloodStained/PB_DT_RandomizerRoomCheck"
        )
    };
}

const TEST_ASSET: &[u8] = include_bytes!(concat!(test_asset!(), ".uasset"));

#[test]
fn data_tables() -> Result<(), Error> {
    let mut asset = Asset::new(Cursor::new(TEST_ASSET), None);
    asset.set_engine_version(EngineVersion::VER_UE4_18);

    asset.parse_data()?;
    shared::verify_binary_equality(TEST_ASSET, None, &mut asset)?;
    assert!(shared::verify_all_exports_parsed(&asset));

    let data_table_export: &mut DataTableExport =
        cast!(Export, DataTableExport, &mut asset.exports[0])
            .expect("First export is not a DataTableExport");

    let first_entry = &mut data_table_export.table.data[0];

    let mut flipped_values = HashMap::new();
    let mut found_test_name = false;
    // flip all the flags for further testing
    for property in &mut first_entry.value {
        let property_name = property.get_name().content;
        if property_name == "AcceleratorANDDoubleJump" {
            found_test_name = true;
        }

        if let Some(bool_prop) = cast!(Property, BoolProperty, property) {
            bool_prop.value = !bool_prop.value;
            flipped_values.insert(property_name, bool_prop.value);
        }
    }
    assert!(found_test_name);

    let mut modified = Cursor::new(Vec::new());
    asset.write_data(&mut modified, None)?;
    let modified = modified.into_inner();

    let mut parsed_back = Asset::new(Cursor::new(modified.as_slice()), None);
    parsed_back.set_engine_version(EngineVersion::VER_UE4_18);

    parsed_back.parse_data()?;
    shared::verify_binary_equality(&modified, None, &mut asset)?;
    assert!(shared::verify_all_exports_parsed(&parsed_back));
    assert!(parsed_back.exports.len() == 1);

    let data_table_export: &DataTableExport =
        cast!(Export, DataTableExport, &parsed_back.exports[0])
            .expect("First export is not a DataTableExport after serializing and deserializing");

    let first_entry = &data_table_export.table.data[0];

    for property in &first_entry.value {
        if let Some(bool_prop) = cast!(Property, BoolProperty, property) {
            assert_eq!(
                *flipped_values.get(&bool_prop.get_name().content).unwrap(),
                bool_prop.value
            );
        }
    }

    Ok(())
}
