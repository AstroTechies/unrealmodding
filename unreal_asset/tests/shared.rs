use std::io::Cursor;

use unreal_asset::{cast, error::Error, exports::Export, reader::asset_trait::AssetTrait, Asset};

#[allow(dead_code)]
pub(crate) fn verify_reparse(asset: &mut Asset) -> Result<(), Error> {
    let engine_version = asset.get_engine_version();

    let mut cursor = Cursor::new(Vec::new());
    let mut bulk_cursor = Cursor::new(Vec::new());
    asset.write_data(&mut cursor, Some(&mut bulk_cursor))?;

    let mut reparse = Asset::new(cursor.into_inner(), Some(bulk_cursor.into_inner()));
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
