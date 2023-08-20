//! Float range property

use crate::property_prelude::*;

/// Float range property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FloatRangeProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Lower bound
    pub lower_bound: OrderedFloat<f32>,
    /// Upper bound
    pub upper_bound: OrderedFloat<f32>,
}
impl_property_data_trait!(FloatRangeProperty);

impl FloatRangeProperty {
    /// Read a `FloatRangeProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let lower_bound = asset.read_f32::<LE>()?;
        let upper_bound = asset.read_f32::<LE>()?;

        Ok(FloatRangeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            lower_bound: OrderedFloat(lower_bound),
            upper_bound: OrderedFloat(upper_bound),
        })
    }
}

impl PropertyTrait for FloatRangeProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        asset.write_f32::<LE>(self.lower_bound.0)?;
        asset.write_f32::<LE>(self.upper_bound.0)?;

        Ok((asset.position() - begin) as usize)
    }
}
