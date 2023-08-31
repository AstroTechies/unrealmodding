//! World export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    reader::{ArchiveReader, ArchiveWriter},
    types::{PackageIndex, PackageIndexTrait},
    Error, FNameContainer,
};

use crate::implement_get;
use crate::ExportTrait;
use crate::{BaseExport, NormalExport};

/// World export
///
/// This is a `World` export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldExport<Index: PackageIndexTrait> {
    /// Base normal export
    pub normal_export: NormalExport<Index>,
    /// Persistent level - a LevelExport
    #[container_ignore]
    pub persistent_level: PackageIndex,
    /// Extra objects
    #[container_ignore]
    pub extra_objects: Vec<PackageIndex>,
    /// Levels streaming in the world
    #[container_ignore]
    pub streaming_levels: Vec<PackageIndex>,
}

implement_get!(WorldExport);

impl<Index: PackageIndexTrait> WorldExport<Index> {
    /// Read a `WorldExport` from an asset
    pub fn from_base<Reader: ArchiveReader<Index>>(
        base: &BaseExport<Index>,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;
        asset.read_i32::<LE>()?;
        Ok(WorldExport {
            normal_export,
            persistent_level: PackageIndex::new(asset.read_i32::<LE>()?),
            extra_objects: asset
                .read_array(|asset| Ok(PackageIndex::new(asset.read_i32::<LE>()?)))?,
            streaming_levels: asset
                .read_array(|asset| Ok(PackageIndex::new(asset.read_i32::<LE>()?)))?,
        })
    }
}

impl<Index: PackageIndexTrait> ExportTrait<Index> for WorldExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;

        asset.write_i32::<LE>(0)?;

        asset.write_i32::<LE>(self.persistent_level.index)?;

        asset.write_i32::<LE>(self.extra_objects.len() as i32)?;
        for object in &self.extra_objects {
            asset.write_i32::<LE>(object.index)?;
        }

        asset.write_i32::<LE>(self.streaming_levels.len() as i32)?;
        for level in &self.streaming_levels {
            asset.write_i32::<LE>(level.index)?;
        }

        Ok(())
    }
}
