use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

use super::PropertyTrait;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatRangeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub lower_bound: OrderedFloat<f32>,
    pub upper_bound: OrderedFloat<f32>,
}
impl_property_data_trait!(FloatRangeProperty);

impl FloatRangeProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let lower_bound = asset.read_f32::<LittleEndian>()?;
        let upper_bound = asset.read_f32::<LittleEndian>()?;

        Ok(FloatRangeProperty {
            name,
            property_guid,
            duplication_index,
            lower_bound: OrderedFloat(lower_bound),
            upper_bound: OrderedFloat(upper_bound),
        })
    }
}

impl PropertyTrait for FloatRangeProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        asset.write_f32::<LittleEndian>(self.lower_bound.0)?;
        asset.write_f32::<LittleEndian>(self.upper_bound.0)?;

        Ok((asset.position() - begin) as usize)
    }
}
