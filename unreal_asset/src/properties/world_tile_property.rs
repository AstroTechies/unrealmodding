use byteorder::LittleEndian;

use crate::custom_version::FFortniteMainBranchObjectVersion;
use crate::error::Error;
use crate::object_version::ObjectVersion;
use crate::properties::{
    vector_property::{BoxProperty, IntPointProperty},
    PropertyTrait,
};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::vector::Vector;
use crate::types::FName;

//todo: what is this file even doing in properties?
#[derive(Clone)]
pub struct FWorldTileLayer {
    pub name: Option<String>,
    pub reserved_0: i32,
    pub reserved_1: IntPointProperty,
    pub streaming_distance: Option<i32>,
    pub distance_streaming_enabled: Option<bool>,
}

impl FWorldTileLayer {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let object_version = asset.get_object_version();

        let name = asset.read_string()?;
        let reserved_0 = asset.read_i32::<LittleEndian>()?;
        let reserved_1 = IntPointProperty::new(asset, FName::default(), false, 0)?;

        let streaming_distance =
            match object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_UPDATED {
                true => Some(asset.read_i32::<LittleEndian>()?),
                false => None,
            };

        let distance_streaming_enabled =
            match object_version >= ObjectVersion::VER_UE4_WORLD_LAYER_ENABLE_DISTANCE_STREAMING {
                true => Some(asset.read_i32::<LittleEndian>()? == 1),
                false => None,
            };

        Ok(FWorldTileLayer {
            name,
            reserved_0,
            reserved_1,
            streaming_distance,
            distance_streaming_enabled,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        let object_version = asset.get_object_version();

        asset.write_string(&self.name)?;
        asset.write_i32::<LittleEndian>(self.reserved_0)?;
        self.reserved_1.write(asset, false)?;

        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_UPDATED {
            asset.write_i32::<LittleEndian>(
                self.streaming_distance
                    .ok_or_else(|| Error::no_data("object_version >= VER_UE4_WORLD_LEVEL_INFO_UPDATED but streaming_distance is None".to_string()))?,
            )?;
        }

        if object_version >= ObjectVersion::VER_UE4_WORLD_LAYER_ENABLE_DISTANCE_STREAMING {
            asset.write_i32::<LittleEndian>(
                match self.distance_streaming_enabled.ok_or_else(|| {
                    Error::no_data(
                        "object_version >= VER_UE4_WORLD_LAYER_ENABLE_DISTANCE_STREAMING but distance_streaming_enabled is None".to_string(),
                    )
                })? {
                    true => 1,
                    false => 0,
                },
            )?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct FWorldTileLODInfo {
    pub relative_streaming_distance: i32,
    pub reserved_0: f32,
    pub reserved_1: f32,
    pub reserved_2: i32,
    pub reserved_3: i32,
}

impl FWorldTileLODInfo {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(FWorldTileLODInfo {
            relative_streaming_distance: asset.read_i32::<LittleEndian>()?,
            reserved_0: asset.read_f32::<LittleEndian>()?,
            reserved_1: asset.read_f32::<LittleEndian>()?,
            reserved_2: asset.read_i32::<LittleEndian>()?,
            reserved_3: asset.read_i32::<LittleEndian>()?,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.relative_streaming_distance)?;
        asset.write_f32::<LittleEndian>(self.reserved_0)?;
        asset.write_f32::<LittleEndian>(self.reserved_1)?;
        asset.write_i32::<LittleEndian>(self.reserved_2)?;
        asset.write_i32::<LittleEndian>(self.reserved_3)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct FWorldTileInfo {
    position: Vector<i32>,
    pub bounds: BoxProperty,
    //absolute_position: Vector<i32>, // not set in most recent version of uassetapi?
    pub layer: FWorldTileLayer,
    pub hide_in_tile_view: Option<bool>,
    pub parent_tile_package_name: Option<String>,
    pub lod_list: Option<Vec<FWorldTileLODInfo>>,
    pub z_order: Option<i32>,
}

impl FWorldTileInfo {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let version = asset.get_custom_version::<FFortniteMainBranchObjectVersion>();
        let object_version = asset.get_object_version();

        let position = match version.version
            < FFortniteMainBranchObjectVersion::WorldCompositionTile3DOffset as i32
        {
            true => Vector::new(
                asset.read_i32::<LittleEndian>()?,
                asset.read_i32::<LittleEndian>()?,
                0,
            ),
            false => Vector::new(
                asset.read_i32::<LittleEndian>()?,
                asset.read_i32::<LittleEndian>()?,
                asset.read_i32::<LittleEndian>()?,
            ),
        };

        let bounds = BoxProperty::new(asset, FName::default(), false, 0)?;
        let layer = FWorldTileLayer::new(asset)?;

        let mut hide_in_tile_view = None;
        let mut parent_tile_package_name = None;
        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_UPDATED {
            hide_in_tile_view = Some(asset.read_i32::<LittleEndian>()? == 1);
            parent_tile_package_name = asset.read_string()?;
        }

        let mut lod_list = None;
        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_LOD_LIST {
            let num_entries = asset.read_i32::<LittleEndian>()? as usize;
            let mut list = Vec::with_capacity(num_entries);
            for _i in 0..num_entries {
                list.push(FWorldTileLODInfo::new(asset)?);
            }
            lod_list = Some(list);
        }

        let z_order = match object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_ZORDER {
            true => Some(asset.read_i32::<LittleEndian>()?),
            false => None,
        };

        Ok(FWorldTileInfo {
            position,
            bounds,
            layer,
            hide_in_tile_view,
            parent_tile_package_name,
            lod_list,
            z_order,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        let object_version = asset.get_object_version();

        if asset
            .get_custom_version::<FFortniteMainBranchObjectVersion>()
            .version
            < FFortniteMainBranchObjectVersion::WorldCompositionTile3DOffset as i32
        {
            asset.write_i32::<LittleEndian>(self.position.x)?;
            asset.write_i32::<LittleEndian>(self.position.y)?;
        } else {
            asset.write_i32::<LittleEndian>(self.position.x)?;
            asset.write_i32::<LittleEndian>(self.position.y)?;
            asset.write_i32::<LittleEndian>(self.position.z)?;
        }

        self.bounds.write(asset, false)?;
        self.layer.write(asset)?;

        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_UPDATED {
            asset.write_i32::<LittleEndian>(
                match self
                    .hide_in_tile_view
                    .ok_or_else(|| Error::no_data("object_version >= VER_UE4_WORLD_LEVEL_INFO_UPDATED but hide_in_tile_view is None".to_string()))?
                {
                    true => 1,
                    false => 0,
                },
            )?;

            asset.write_string(&self.parent_tile_package_name)?;
        }

        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_LOD_LIST {
            let lod_list = self.lod_list.as_ref().ok_or_else(|| {
                Error::no_data(
                    "object_version >= VER_UE4_WORLD_LEVEL_INFO_LOD_LIST but lod_list is None"
                        .to_string(),
                )
            })?;

            asset.write_i32::<LittleEndian>(lod_list.len() as i32)?;
            for entry in lod_list {
                entry.write(asset)?;
            }
        }

        if object_version >= ObjectVersion::VER_UE4_WORLD_LEVEL_INFO_ZORDER {
            asset.write_i32::<LittleEndian>(self.z_order.ok_or_else(|| {
                Error::no_data(
                    "object_version >= VER_UE4_WORLD_LEVEL_INFO_ZORDER but z_order is None"
                        .to_string(),
                )
            })?)?;
        }

        Ok(())
    }
}
