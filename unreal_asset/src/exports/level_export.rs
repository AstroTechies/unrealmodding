//! Level export

use byteorder::LittleEndian;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::exports::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::implement_get;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::PackageIndex;

/// Level export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LevelExport {
    /// Base normal export
    pub normal_export: NormalExport,

    /// Level actors
    #[container_ignore]
    pub actors: Vec<PackageIndex>,
    /// Level namespace
    pub namespace: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Flags?
    pub flags_probably: u64,
    /// Misc category data
    #[container_ignore]
    pub misc_category_data: Vec<PackageIndex>,
}

implement_get!(LevelExport);

impl LevelExport {
    /// Read a `LevelExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        unk: &BaseExport,
        asset: &mut Reader,
        next_starting: u64,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(unk, asset)?;

        asset.read_i32::<LittleEndian>()?;

        let num_actors = asset.read_i32::<LittleEndian>()?;
        let mut actors = Vec::with_capacity(num_actors as usize);
        for _i in 0..num_actors as usize {
            actors.push(PackageIndex::new(asset.read_i32::<LittleEndian>()?));
        }

        let namespace = asset.read_fstring()?;
        asset.read_i32::<LittleEndian>()?; // null
        let value = asset.read_fstring()?;

        asset.read_i64::<LittleEndian>()?; // null
        let flags_probably = asset.read_u64::<LittleEndian>()?;
        let mut misc_category_data = Vec::new();
        while asset.position() < next_starting - 1 {
            misc_category_data.push(PackageIndex::new(asset.read_i32::<LittleEndian>()?));
        }
        asset.read_exact(&mut [0u8; 1])?;

        Ok(LevelExport {
            normal_export,
            actors,
            namespace,
            value,
            flags_probably,
            misc_category_data,
        })
    }
}

impl ExportTrait for LevelExport {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;

        asset.write_i32::<LittleEndian>(0)?;
        asset.write_i32::<LittleEndian>(self.actors.len() as i32)?;
        for actor in &self.actors {
            asset.write_i32::<LittleEndian>(actor.index)?;
        }

        asset.write_fstring(self.namespace.as_deref())?;
        asset.write_i32::<LittleEndian>(0)?;
        asset.write_fstring(self.value.as_deref())?;

        asset.write_u64::<LittleEndian>(0)?;
        asset.write_u64::<LittleEndian>(self.flags_probably)?;

        for data in &self.misc_category_data {
            asset.write_i32::<LittleEndian>(data.index)?;
        }
        asset.write_u8(0)?;
        Ok(())
    }
}
