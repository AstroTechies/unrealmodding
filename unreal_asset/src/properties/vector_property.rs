//! Vector properties

use std::mem::size_of;

use byteorder::LE;
use ordered_float::OrderedFloat;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersionUE5;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::fname::FName;
use crate::types::vector::Plane;
use crate::types::vector::Vector2;
use crate::types::vector::{Vector, Vector4};
use crate::unversioned::ancestry::Ancestry;

/// Vector property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct VectorProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Vector value
    #[container_ignore]
    pub value: Vector<OrderedFloat<f64>>,
}
impl_property_data_trait!(VectorProperty);

/// Int point property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct IntPointProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    #[container_ignore]
    pub value: Vector2<i32>,
}
impl_property_data_trait!(IntPointProperty);

/// Vector4 property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector4Property {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Vector4 value
    #[container_ignore]
    pub value: Vector4<OrderedFloat<f64>>,
}
impl_property_data_trait!(Vector4Property);

/// Vector2D property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector2DProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    #[container_ignore]
    pub value: Vector2<OrderedFloat<f64>>,
}
impl_property_data_trait!(Vector2DProperty);

/// Quaternion property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct QuatProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Quaternion value
    #[container_ignore]
    pub value: Vector4<OrderedFloat<f64>>,
}
impl_property_data_trait!(QuatProperty);

/// Rotator property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct RotatorProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Rotator value
    #[container_ignore]
    pub value: Vector<OrderedFloat<f64>>,
}
impl_property_data_trait!(RotatorProperty);

/// Box property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct BoxProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// First box corner
    pub v1: VectorProperty,
    /// Second box corner
    pub v2: VectorProperty,
    /// Is box valid
    pub is_valid: bool,
}
impl_property_data_trait!(BoxProperty);

/// Box2D property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Box2DProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Top-left box corner
    pub v1: Vector2DProperty,
    /// Bottom-right box corner
    pub v2: Vector2DProperty,
    /// Is box valid
    pub is_valid: bool,
}
impl_property_data_trait!(Box2DProperty);

/// Plane property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PlaneProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    #[container_ignore]
    pub value: Plane<OrderedFloat<f64>>,
}
impl_property_data_trait!(PlaneProperty);

impl VectorProperty {
    /// Read a `VectorProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => Vector::new(
                    OrderedFloat(asset.read_f64::<LE>()?),
                    OrderedFloat(asset.read_f64::<LE>()?),
                    OrderedFloat(asset.read_f64::<LE>()?),
                ),
                false => Vector::new(
                    OrderedFloat(asset.read_f32::<LE>()? as f64),
                    OrderedFloat(asset.read_f32::<LE>()? as f64),
                    OrderedFloat(asset.read_f32::<LE>()? as f64),
                ),
            };
        Ok(VectorProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for VectorProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                asset.write_f64::<LE>(self.value.z.0)?;
                Ok(size_of::<f64>() * 3)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                asset.write_f32::<LE>(self.value.z.0 as f32)?;
                Ok(size_of::<f32>() * 3)
            }
        }
    }
}

impl IntPointProperty {
    /// Read an `IntPointProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let x = asset.read_i32::<LE>()?;
        let y = asset.read_i32::<LE>()?;

        Ok(IntPointProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: Vector2::new(x, y),
        })
    }
}

impl PropertyTrait for IntPointProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LE>(self.value.x)?;
        asset.write_i32::<LE>(self.value.y)?;
        Ok(size_of::<i32>() * 2)
    }
}

impl Vector4Property {
    /// Read a `Vector4Property` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => {
                    let x = OrderedFloat(asset.read_f64::<LE>()?);
                    let y = OrderedFloat(asset.read_f64::<LE>()?);
                    let z = OrderedFloat(asset.read_f64::<LE>()?);
                    let w = OrderedFloat(asset.read_f64::<LE>()?);
                    Vector4::new(x, y, z, w)
                }
                false => {
                    let x = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let y = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let z = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let w = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    Vector4::new(x, y, z, w)
                }
            };
        Ok(Vector4Property {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for Vector4Property {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                asset.write_f64::<LE>(self.value.z.0)?;
                asset.write_f64::<LE>(self.value.w.0)?;
                Ok(size_of::<f64>() * 4)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                asset.write_f32::<LE>(self.value.z.0 as f32)?;
                asset.write_f32::<LE>(self.value.w.0 as f32)?;
                Ok(size_of::<f32>() * 4)
            }
        }
    }
}

impl Vector2DProperty {
    /// Read a `Vector2DProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => {
                    let x = OrderedFloat(asset.read_f64::<LE>()?);
                    let y = OrderedFloat(asset.read_f64::<LE>()?);
                    Vector2::new(x, y)
                }
                false => {
                    let x = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let y = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    Vector2::new(x, y)
                }
            };

        Ok(Vector2DProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for Vector2DProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                Ok(size_of::<f64>() * 2)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                Ok(size_of::<f32>() * 2)
            }
        }
    }
}

impl QuatProperty {
    /// Read a `QuatProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => {
                    let x = OrderedFloat(asset.read_f64::<LE>()?);
                    let y = OrderedFloat(asset.read_f64::<LE>()?);
                    let z = OrderedFloat(asset.read_f64::<LE>()?);
                    let w = OrderedFloat(asset.read_f64::<LE>()?);
                    Vector4::new(x, y, z, w)
                }
                false => {
                    let x = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let y = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let z = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let w = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    Vector4::new(x, y, z, w)
                }
            };

        Ok(QuatProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for QuatProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                asset.write_f64::<LE>(self.value.z.0)?;
                asset.write_f64::<LE>(self.value.w.0)?;
                Ok(size_of::<f64>() * 4)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                asset.write_f32::<LE>(self.value.z.0 as f32)?;
                asset.write_f32::<LE>(self.value.w.0 as f32)?;
                Ok(size_of::<f32>() * 4)
            }
        }
    }
}

impl RotatorProperty {
    /// Read a `RotatorProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => {
                    let x = OrderedFloat(asset.read_f64::<LE>()?);
                    let y = OrderedFloat(asset.read_f64::<LE>()?);
                    let z = OrderedFloat(asset.read_f64::<LE>()?);
                    Vector::new(x, y, z)
                }
                false => {
                    let x = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let y = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let z = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    Vector::new(x, y, z)
                }
            };

        Ok(RotatorProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for RotatorProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                asset.write_f64::<LE>(self.value.z.0)?;

                Ok(size_of::<f64>() * 3)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                asset.write_f32::<LE>(self.value.z.0 as f32)?;

                Ok(size_of::<f32>() * 3)
            }
        }
    }
}

impl BoxProperty {
    /// Read a `BoxProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = match include_header {
            true => asset.read_property_guid()?,
            false => None,
        };

        let new_ancestry = ancestry.with_parent(name.clone());
        let v1 = VectorProperty::new(asset, name.clone(), new_ancestry.clone(), false, 0)?;
        let v2 = VectorProperty::new(asset, name.clone(), new_ancestry, false, 0)?;
        let is_valid = asset.read_bool()?;

        Ok(BoxProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            v1,
            v2,
            is_valid,
        })
    }
}

impl PropertyTrait for BoxProperty {
    fn write<Writer: ArchiveWriter>(
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
    /// Read a `Box2DProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let new_ancestry = ancestry.with_parent(name.clone());
        Ok(Box2DProperty {
            name: name.clone(),
            ancestry,
            property_guid,
            duplication_index,
            v1: Vector2DProperty::new(asset, name.clone(), new_ancestry.clone(), false, 0)?,
            v2: Vector2DProperty::new(asset, name, new_ancestry, false, 0)?,
            is_valid: asset.read_bool()?,
        })
    }
}

impl PropertyTrait for Box2DProperty {
    fn write<Writer: ArchiveWriter>(
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

impl PlaneProperty {
    /// Read a `PlaneProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value =
            match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
                true => {
                    let x = OrderedFloat(asset.read_f64::<LE>()?);
                    let y = OrderedFloat(asset.read_f64::<LE>()?);
                    let z = OrderedFloat(asset.read_f64::<LE>()?);
                    let w = OrderedFloat(asset.read_f64::<LE>()?);

                    Plane::new(x, y, z, w)
                }
                false => {
                    let x = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let y = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let z = OrderedFloat(asset.read_f32::<LE>()? as f64);
                    let w = OrderedFloat(asset.read_f32::<LE>()? as f64);

                    Plane::new(x, y, z, w)
                }
            };

        Ok(PlaneProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PlaneProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        match asset.get_object_version_ue5() >= ObjectVersionUE5::LARGE_WORLD_COORDINATES {
            true => {
                asset.write_f64::<LE>(self.value.x.0)?;
                asset.write_f64::<LE>(self.value.y.0)?;
                asset.write_f64::<LE>(self.value.z.0)?;
                asset.write_f64::<LE>(self.value.w.0)?;

                Ok(size_of::<f64>() * 4)
            }
            false => {
                asset.write_f32::<LE>(self.value.x.0 as f32)?;
                asset.write_f32::<LE>(self.value.y.0 as f32)?;
                asset.write_f32::<LE>(self.value.z.0 as f32)?;
                asset.write_f32::<LE>(self.value.w.0 as f32)?;

                Ok(size_of::<f32>() * 4)
            }
        }
    }
}
