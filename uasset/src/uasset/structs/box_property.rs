use std::io::{Cursor, Error};

use crate::uasset::{unreal_types::Guid, cursor_ext::CursorExt};

use super::vector_property::VectorProperty;

#[derive(Debug)]
pub struct BoxProperty {
    property_guid: Option<Guid>,
    v1: VectorProperty,
    v2: VectorProperty,
    is_valid: bool
}

impl BoxProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => Some(cursor.read_property_guid()?),
            false => None
        };

        Ok(BoxProperty {
            property_guid,
            v1: VectorProperty::new(cursor, false)?,
            v2: VectorProperty::new(cursor, false)?,
            is_valid: cursor.read_bool()?
        })
    }
}