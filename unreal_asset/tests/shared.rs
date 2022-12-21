use std::fs::File;
use std::io::Cursor;
use std::io::Write;

use unreal_asset::{cast, engine_version::EngineVersion, error::Error, exports::Export, Asset};

#[allow(dead_code)]
pub(crate) fn verify_reparse(
    asset: &mut Asset,
    engine_version: EngineVersion,
) -> Result<(), Error> {
    let mut cursor = Cursor::new(Vec::new());

    let mut bulk_cursor = None;
    if asset.use_separate_bulk_data_files {
        bulk_cursor = Some(Cursor::new(Vec::new()));
    }
    asset.write_data(&mut cursor, bulk_cursor.as_mut())?;

    let cursor_inner = cursor.into_inner();
    {
        let mut file = File::create("C:\\Users\\Kate\\Desktop\\astro\\test_thing.uasset")?;
        file.write_all(&cursor_inner)?;
    }

    let bulk_inner = bulk_cursor.map(|e| e.into_inner());
    if let Some(ref bulk_inner) = bulk_inner {
        let mut file = File::create("C:\\Users\\Kate\\Desktop\\astro\\test_thing.uexp")?;
        file.write_all(&bulk_inner)?;
    }

    let mut reparse = Asset::new(cursor_inner, bulk_inner);
    reparse.set_engine_version(engine_version);

    reparse.parse_data()?;

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
    if asset.use_separate_bulk_data_files {
        bulk_cursor = Some(Cursor::new(Vec::new()));
    }
    asset.write_data(&mut cursor, bulk_cursor.as_mut())?;

    if bulk.is_some() != bulk_cursor.is_some() {
        panic!("Invalid check binary equality params");
    }

    let cursor_inner = cursor.into_inner();
    {
        let mut file = File::create("C:\\Users\\Kate\\Desktop\\astro\\test_thing.uasset")?;
        file.write_all(&cursor_inner)?;
    }

    let bulk_inner = bulk_cursor.map(|e| e.into_inner());
    if let Some(ref bulk_inner) = bulk_inner {
        let mut file = File::create("C:\\Users\\Kate\\Desktop\\astro\\test_thing.uexp")?;
        file.write_all(&bulk_inner)?;
    }

    assert_eq!(cursor_inner, data);

    if let Some(bulk_cursor) = bulk_inner {
        assert_eq!(bulk_cursor, bulk.unwrap());
    }

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn verify_all_exports_parsed(asset: &Asset) -> bool {
    for export in &asset.exports {
        if cast!(Export, RawExport, export).is_some() {
            return false;
        }
    }

    true
}
