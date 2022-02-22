pub mod int_property;
pub mod str_property;
pub mod object_property;
pub mod guid_property;
pub mod date_property;
pub mod color_property;
pub mod vector_property;

use std::io::{Error, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use enum_dispatch::enum_dispatch;

use super::Asset;

#[macro_export]
macro_rules! optional_guid {
    ($cursor:ident, $include_header:ident) => {
        match $include_header {
            true => Some($cursor.read_property_guid()?),
            false => None
        };
    };
}

#[enum_dispatch]
trait PropertyTrait {
}

#[enum_dispatch(PropertyTrait)]
pub enum Property {

}

impl Property {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: Asset, include_header: bool) -> Result<Option<Self>, Error> {
        let offset = cursor.position();
        let name = asset.read_fname()?; // probably should pass cursor instancce there
        if &name.content == "None" {
            return Ok(None);
        }

        let property_type = asset.read_fname()?; // probably should pass cursor instance there
        let length = cursor.read_i32::<LittleEndian>()?;
        let duplication_index = cursor.read_i32::<LittleEndian>()?;

        Ok(Some(Property{}))
    }
}