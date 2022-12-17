use std::io::Cursor;

use unreal_asset::{error::Error, reader::asset_trait::AssetTrait, Asset};

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
