use std::io::Cursor;

use unreal_asset::{cast, error::Error, exports::Export, reader::asset_trait::AssetTrait, Asset};

#[allow(dead_code)]
pub(crate) fn verify_reparse(asset: &mut Asset) -> Result<(), Error> {
    let engine_version = asset.get_engine_version();

    let mut cursor = Cursor::new(Vec::new());

    let mut bulk_cursor = None;
    if asset.use_separate_bulk_data_files {
        bulk_cursor = Some(Cursor::new(Vec::new()));
    }
    asset.write_data(&mut cursor, bulk_cursor.as_mut())?;

    let mut reparse = Asset::new(cursor.into_inner(), bulk_cursor.map(|e| e.into_inner()));
    reparse.set_engine_version(engine_version);

    asset.parse_data()?;

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