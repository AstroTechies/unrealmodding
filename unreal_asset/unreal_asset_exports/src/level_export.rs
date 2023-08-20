//! Level export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndex,
    Error, FNameContainer,
};

use crate::implement_get;
use crate::ExportTrait;
use crate::{BaseExport, NormalExport};

/// Level URL info
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct URL {
    /// Level protocol like "unreal" or "http"
    pub protocol: Option<String>,
    /// Host name like "unreal.epicgames" or "168.192.1.1"
    pub host: Option<String>,
    /// Name of the map
    pub map: Option<String>,
    /// Portal to enter through
    pub portal: Option<String>,
    /// Options
    pub options: Vec<Option<String>>,
    /// Host port
    pub port: i32,
    /// is valid?
    pub valid: i32,
}

/// Level export
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct LevelExport {
    /// Base normal export
    pub normal_export: NormalExport,

    /// Level URL info
    pub url: URL,
    /// Level actors
    #[container_ignore]
    pub actors: Vec<PackageIndex>,
    /// Model export reference
    #[container_ignore]
    pub model: PackageIndex,
    /// Model component references
    #[container_ignore]
    pub model_components: Vec<PackageIndex>,
    /// Level script reference
    #[container_ignore]
    pub level_script: PackageIndex,
    /// start of the navigation component list
    #[container_ignore]
    pub nav_list_start: PackageIndex,
    /// end of the navigation component list
    #[container_ignore]
    pub nav_list_end: PackageIndex,
}

implement_get!(LevelExport);

impl LevelExport {
    /// Read a `LevelExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        unk: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(unk, asset)?;
        asset.read_i32::<LE>()?;

        Ok(LevelExport {
            normal_export,
            actors: asset.read_array(|asset| Ok(PackageIndex::new(asset.read_i32::<LE>()?)))?,
            url: URL {
                protocol: asset.read_fstring()?,
                host: asset.read_fstring()?,
                map: asset.read_fstring()?,
                portal: asset.read_fstring()?,
                options: asset.read_array(|asset| asset.read_fstring())?,
                port: asset.read_i32::<LE>()?,
                valid: asset.read_i32::<LE>()?,
            },
            model: PackageIndex::new(asset.read_i32::<LE>()?),
            model_components: asset
                .read_array(|asset| Ok(PackageIndex::new(asset.read_i32::<LE>()?)))?,
            level_script: PackageIndex::new(asset.read_i32::<LE>()?),
            nav_list_start: PackageIndex::new(asset.read_i32::<LE>()?),
            nav_list_end: PackageIndex::new(asset.read_i32::<LE>()?),
        })
    }
}

impl ExportTrait for LevelExport {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;

        asset.write_i32::<LE>(0)?;
        asset.write_i32::<LE>(self.actors.len() as i32)?;
        for actor in &self.actors {
            asset.write_i32::<LE>(actor.index)?;
        }

        asset.write_fstring(self.url.protocol.as_deref())?;
        asset.write_fstring(self.url.host.as_deref())?;
        asset.write_fstring(self.url.map.as_deref())?;
        asset.write_fstring(self.url.portal.as_deref())?;

        asset.write_i32::<LE>(self.url.options.len() as i32)?;
        for option in &self.url.options {
            asset.write_fstring(option.as_deref())?;
        }

        asset.write_i32::<LE>(self.url.port)?;
        asset.write_i32::<LE>(self.url.valid)?;

        asset.write_i32::<LE>(self.model.index)?;

        asset.write_i32::<LE>(self.model_components.len() as i32)?;
        for data in &self.model_components {
            asset.write_i32::<LE>(data.index)?;
        }
        asset.write_i32::<LE>(self.level_script.index)?;
        asset.write_i32::<LE>(self.nav_list_start.index)?;
        asset.write_i32::<LE>(self.nav_list_end.index)?;
        Ok(())
    }
}
