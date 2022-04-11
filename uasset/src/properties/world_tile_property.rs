use std::{io::{Cursor}};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::uasset::{types::Vector, cursor_ext::CursorExt, ue4version::{VER_UE4_WORLD_LEVEL_INFO_UPDATED, VER_UE4_WORLD_LAYER_ENABLE_DISTANCE_STREAMING, VER_UE4_WORLD_LEVEL_INFO_LOD_LIST, VER_UE4_WORLD_LEVEL_INFO_ZORDER}, Asset, custom_version::FFortniteMainBranchObjectVersion, unreal_types::FName};
use crate::uasset::error::Error;
use super::vector_property::{IntPointProperty, BoxProperty};

//todo: what is this file even doing in properties?

pub struct FWorldTileLayer {
    pub name: Option<String>,
    pub reserved_0: i32,
    pub reserved_1: IntPointProperty,
    pub streaming_distance: Option<i32>,
    pub distance_streaming_enabled: Option<bool>
}

impl FWorldTileLayer {
    pub fn new(asset: &mut Asset, engine_version: i32) -> Result<Self, Error> {
        let name = asset.cursor.read_string()?;
        let reserved_0 = asset.cursor.read_i32::<LittleEndian>()?;
        let reserved_1 = IntPointProperty::new(asset, FName::default(), false, 0)?;
        
        let streaming_distance = match engine_version >= VER_UE4_WORLD_LEVEL_INFO_UPDATED {
            true => Some(asset.cursor.read_i32::<LittleEndian>()?),
            false => None
        };

        let distance_streaming_enabled = match engine_version >= VER_UE4_WORLD_LAYER_ENABLE_DISTANCE_STREAMING {
            true => Some(asset.cursor.read_i32::<LittleEndian>()? == 1),
            false => None
        };

        Ok(FWorldTileLayer {
            name, reserved_0, reserved_1, streaming_distance, distance_streaming_enabled
        })
    }
}

pub struct FWorldTileLODInfo {
    pub relative_streaming_distance: i32,
    pub reserved_0: f32,
    pub reserved_1: f32,
    pub reserved_2: i32,
    pub reserved_3: i32
}

impl FWorldTileLODInfo {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(FWorldTileLODInfo {
            relative_streaming_distance: asset.cursor.read_i32::<LittleEndian>()?,
            reserved_0: asset.cursor.read_f32::<LittleEndian>()?,
            reserved_1: asset.cursor.read_f32::<LittleEndian>()?,
            reserved_2: asset.cursor.read_i32::<LittleEndian>()?,
            reserved_3: asset.cursor.read_i32::<LittleEndian>()?
        })
    }
}

pub struct FWorldTileInfo {
    position: Vector<i32>,
    pub bounds: BoxProperty,
    //absolute_position: Vector<i32>, // not set in most recent version of uassetapi?
    pub layer: FWorldTileLayer,
    pub hide_in_tile_view: Option<bool>,
    pub parent_tile_package_name: Option<String>,
    pub lod_list: Option<Vec<FWorldTileLODInfo>>,
    pub z_order: Option<i32>
}

impl FWorldTileInfo {
    pub fn new(asset: &mut Asset, engine_version: i32) -> Result<Self, Error> {
        let version = asset.get_custom_version::<FFortniteMainBranchObjectVersion>();

        let position = match version.version < FFortniteMainBranchObjectVersion::WorldCompositionTile3DOffset as i32 {
            true => Vector::new(asset.cursor.read_i32::<LittleEndian>()?, asset.cursor.read_i32::<LittleEndian>()?, 0),
            false => asset.cursor.read_int_vector()?
        };

        let bounds = BoxProperty::new(asset, FName::default(), false, 0)?;
        let layer = FWorldTileLayer::new(asset, engine_version)?;

        let mut hide_in_tile_view = None;
        let mut parent_tile_package_name = None;
        if engine_version >= VER_UE4_WORLD_LEVEL_INFO_UPDATED {
            hide_in_tile_view = Some(asset.cursor.read_i32::<LittleEndian>()? == 1);
            parent_tile_package_name = asset.cursor.read_string()?;
        }

        let mut lod_list = None;
        if engine_version >= VER_UE4_WORLD_LEVEL_INFO_LOD_LIST {
            let num_entries = asset.cursor.read_i32::<LittleEndian>()? as usize;
            let mut list = Vec::with_capacity(num_entries);
            for _i in 0..num_entries {
                list.push(FWorldTileLODInfo::new(asset)?);
            }
            lod_list = Some(list);
        }

        let z_order = match engine_version >= VER_UE4_WORLD_LEVEL_INFO_ZORDER {
            true => Some(asset.cursor.read_i32::<LittleEndian>()?),
            false => None
        };

        Ok(FWorldTileInfo {
            position, bounds, layer, hide_in_tile_view, parent_tile_package_name, lod_list, z_order
        })
    }

    pub fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        if asset.get_custom_version::<FFortniteMainBranchObjectVersion>().version < FFortniteMainBranchObjectVersion::WorldCompositionTile3DOffset as i32 {
            cursor.write_i32::<LittleEndian>(self.position.x)?;
            cursor.write_i32::<LittleEndian>(self.position.y)?;
        } else {
            cursor.write_i32::<LittleEndian>(self.position.x)?;
            cursor.write_i32::<LittleEndian>(self.position.y)?;
            cursor.write_i32::<LittleEndian>(self.position.z)?;
        }


        Ok(())
    }
}
