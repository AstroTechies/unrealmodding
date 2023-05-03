use std::io::Cursor;

use unreal_asset::{cast, engine_version::EngineVersion, error::Error, exports::Export, Asset};

#[allow(dead_code)]
pub(crate) fn verify_reparse(
    asset: &mut Asset,
    engine_version: EngineVersion,
) -> Result<(), Error> {
    let mut cursor = Cursor::new(Vec::new());

    let mut bulk_cursor = None;
    if asset.asset_data.use_event_driven_loader {
        bulk_cursor = Some(Cursor::new(Vec::new()));
    }
    asset.write_data(&mut cursor, bulk_cursor.as_mut())?;

    Asset::new(cursor, bulk_cursor, engine_version)?;

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn verify_binary_equality(
    data: &[u8],
    bulk: Option<&[u8]>,
    asset: &mut Asset,
) -> Result<(), Error> {
    let mut cursor = Cursor::new(Vec::new());

    let mut bulk_cursor = None;
    if asset.asset_data.use_event_driven_loader {
        bulk_cursor = Some(Cursor::new(Vec::new()));
    }
    asset.write_data(&mut cursor, bulk_cursor.as_mut())?;

    if bulk.is_some() != bulk_cursor.is_some() {
        panic!("Invalid check binary equality params");
    }

    let cursor_inner = cursor.into_inner();

    let bulk_inner = bulk_cursor.map(|e| e.into_inner());

    assert_eq!(cursor_inner, data);

    if let Some(bulk_cursor) = bulk_inner {
        assert_eq!(bulk_cursor, bulk.unwrap());
    }

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn verify_all_exports_parsed(asset: &Asset) -> bool {
    for export in &asset.asset_data.exports {
        if cast!(Export, RawExport, export).is_some() {
            return false;
        }
    }

    true
}
