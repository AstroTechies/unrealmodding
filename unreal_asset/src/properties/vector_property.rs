use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{Vector, Vector4};
use crate::unreal_types::{FName, Guid};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VectorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vector<OrderedFloat<f32>>,
}
impl_property_data_trait!(VectorProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IntPointProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub x: i32,
    pub y: i32,
}
impl_property_data_trait!(IntPointProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector4Property {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vector4<OrderedFloat<f32>>,
}
impl_property_data_trait!(Vector4Property);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector2DProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}
impl_property_data_trait!(Vector2DProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QuatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vector4<OrderedFloat<f32>>,
}
impl_property_data_trait!(QuatProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RotatorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vector<OrderedFloat<f32>>,
}
impl_property_data_trait!(RotatorProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BoxProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub v1: VectorProperty,
    pub v2: VectorProperty,
    pub is_valid: bool,
}
impl_property_data_trait!(BoxProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Box2DProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub v1: Vector2DProperty,
    pub v2: Vector2DProperty,
    pub is_valid: bool,
}
impl_property_data_trait!(Box2DProperty);

impl VectorProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = Vector::new(
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
        );
        Ok(VectorProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for VectorProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.value.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.z.0)?;
        Ok(size_of::<f32>() * 3)
    }
}

impl IntPointProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let x = asset.read_i32::<LittleEndian>()?;
        let y = asset.read_i32::<LittleEndian>()?;

        Ok(IntPointProperty {
            name,
            property_guid,
            duplication_index,
            x,
            y,
        })
    }
}

impl PropertyTrait for IntPointProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.x)?;
        asset.write_i32::<LittleEndian>(self.y)?;
        Ok(size_of::<i32>() * 2)
    }
}

impl Vector4Property {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let x = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let value = Vector4::new(x, y, z, w);
        Ok(Vector4Property {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for Vector4Property {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.value.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.z.0)?;
        asset.write_f32::<LittleEndian>(self.value.w.0)?;
        Ok(size_of::<f32>() * 4)
    }
}

impl Vector2DProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let x = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.read_f32::<LittleEndian>()?);

        Ok(Vector2DProperty {
            name,
            property_guid,
            duplication_index,
            x,
            y,
        })
    }
}

impl PropertyTrait for Vector2DProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.x.0)?;
        asset.write_f32::<LittleEndian>(self.y.0)?;
        Ok(size_of::<f32>() * 2)
    }
}

impl QuatProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let x = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let w = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let value = Vector4::new(x, y, z, w);

        Ok(QuatProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for QuatProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.value.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.z.0)?;
        asset.write_f32::<LittleEndian>(self.value.w.0)?;
        Ok(size_of::<f32>() * 4)
    }
}

impl RotatorProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let x = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let y = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let z = OrderedFloat(asset.read_f32::<LittleEndian>()?);
        let value = Vector::new(x, y, z);

        Ok(RotatorProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for RotatorProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.value.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.z.0)?;
        Ok(size_of::<f32>() * 3)
    }
}

impl BoxProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => asset.read_property_guid()?,
            false => None,
        };

        let v1 = VectorProperty::new(asset, name.clone(), false, 0)?;
        let v2 = VectorProperty::new(asset, name.clone(), false, 0)?;
        let is_valid = asset.read_bool()?;

        Ok(BoxProperty {
            name,
            property_guid,
            duplication_index,
            v1,
            v2,
            is_valid,
        })
    }
}

impl PropertyTrait for BoxProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let total_size =
            self.v1.write(asset, include_header)? + self.v2.write(asset, include_header)?;
        asset.write_bool(self.is_valid)?;
        Ok(total_size + size_of::<bool>())
    }
}

impl Box2DProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(Box2DProperty {
            name: name.clone(),
            property_guid,
            duplication_index,
            v1: Vector2DProperty::new(asset, name.clone(), false, 0)?,
            v2: Vector2DProperty::new(asset, name, false, 0)?,
            is_valid: asset.read_bool()?,
        })
    }
}

impl PropertyTrait for Box2DProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let total_size =
            self.v1.write(asset, include_header)? + self.v2.write(asset, include_header)?;

        asset.write_bool(self.is_valid)?;
        Ok(total_size + size_of::<bool>())
    }
}
