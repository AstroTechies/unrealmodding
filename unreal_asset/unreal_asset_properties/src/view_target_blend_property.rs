//! View target blend property

use crate::property_prelude::*;

/// View target blend function
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ViewTargetBlendFunction {
    /// Camera does a simple linear interpolation.
    VtBlendLinear,
    /// Camera has a slight ease in and ease out, but amount of ease cannot be tweaked.
    VtBlendCubic,
    /// Camera immediately accelerates, but smoothly decelerates into the target.  Ease amount controlled by BlendExp.
    VtBlendEaseIn,
    /// Camera smoothly accelerates, but does not decelerate into the target.  Ease amount controlled by BlendExp.
    VtBlendEaseOut,
    /// Camera smoothly accelerates and decelerates.  Ease amount controlled by BlendExp.
    VtBlendEaseInOut,
    /// Max
    VtBlendMax,
}

/// View target blend params property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ViewTargetBlendParamsProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Blend time
    pub blend_time: OrderedFloat<f32>,
    /// Blend function
    #[container_ignore]
    pub blend_function: ViewTargetBlendFunction,
    /// Blend exponent
    pub blend_exp: OrderedFloat<f32>,
    /// Lock outgoing
    pub lock_outgoing: bool,
}
impl_property_data_trait!(ViewTargetBlendParamsProperty);

impl ViewTargetBlendParamsProperty {
    /// Read a `ViewTargetBlendParamsProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let blend_time = OrderedFloat(asset.read_f32::<LE>()?);
        let blend_function = ViewTargetBlendFunction::try_from(asset.read_u8()?)?;
        let blend_exp = OrderedFloat(asset.read_f32::<LE>()?);
        let lock_outgoing = asset.read_i32::<LE>()? != 0;

        Ok(ViewTargetBlendParamsProperty {
            name,
            property_guid,
            ancestry,
            duplication_index,
            blend_time,
            blend_function,
            blend_exp,
            lock_outgoing,
        })
    }
}

impl PropertyTrait for ViewTargetBlendParamsProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        asset.write_f32::<LE>(self.blend_time.0)?;
        asset.write_u8(self.blend_function.into())?;
        asset.write_f32::<LE>(self.blend_exp.0)?;
        asset.write_i32::<LE>(match self.lock_outgoing {
            true => 1,
            false => 0,
        })?;
        Ok(size_of::<f32>() * 2 + size_of::<u8>() + size_of::<i32>())
    }
}
