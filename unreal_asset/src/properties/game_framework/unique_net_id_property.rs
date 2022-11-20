use std::{io::SeekFrom, mem::size_of};

use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::asset_reader::AssetReader,
    unreal_types::{FName, Guid},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UniqueNetId {
    pub ty: FName,
    pub contents: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UniqueNetIdProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<UniqueNetId>,
}
impl_property_data_trait!(UniqueNetIdProperty);

impl UniqueNetIdProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let size = asset.read_i32::<LittleEndian>()?;
        let value = match size > 0 {
            true => Some(UniqueNetId {
                ty: asset.read_fname()?,
                contents: asset.read_string()?,
            }),
            false => None,
        };

        Ok(UniqueNetIdProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for UniqueNetIdProperty {
    fn write<Writer: crate::reader::asset_writer::AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        match &self.value {
            Some(value) => {
                let mut length = 3 * size_of::<i32>();
                asset.write_fname(&value.ty)?;
                length += asset.write_string(&value.contents)?;

                let end = asset.position();
                asset.seek(SeekFrom::Start(begin))?;
                asset.write_i32::<LittleEndian>(length as i32)?;
                asset.seek(SeekFrom::Start(end))?;
            }
            None => {
                asset.write_i32::<LittleEndian>(0)?;
            }
        }

        Ok((asset.position() - begin) as usize)
    }
}
