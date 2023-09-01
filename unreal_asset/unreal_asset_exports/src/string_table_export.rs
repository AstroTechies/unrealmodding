//! String table export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    containers::IndexedMap,
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error, FNameContainer,
};

use crate::implement_get;
use crate::ExportTrait;
use crate::{BaseExport, NormalExport};

/// String table export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
pub struct StringTableExport<Index: PackageIndexTrait> {
    /// Base normal export
    pub normal_export: NormalExport<Index>,
    /// String table namespace
    pub namespace: Option<String>,
    /// String table
    pub table: IndexedMap<String, String>,
}

implement_get!(StringTableExport);

impl<Index: PackageIndexTrait> StringTableExport<Index> {
    /// Read a `StringTableExport` from an asset
    pub fn from_base<Reader: ArchiveReader<Index>>(
        base: &BaseExport<Index>,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;
        asset.read_i32::<LE>()?;

        let namespace = asset.read_fstring()?;

        let mut table = IndexedMap::new();
        let num_entries = asset.read_i32::<LE>()?;
        for _ in 0..num_entries {
            table.insert(
                asset
                    .read_fstring()?
                    .ok_or_else(|| Error::no_data("StringTable key is None".to_string()))?,
                asset
                    .read_fstring()?
                    .ok_or_else(|| Error::no_data("StringTable value is None".to_string()))?,
            );
        }

        Ok(StringTableExport {
            normal_export,
            namespace,
            table,
        })
    }
}

impl<Index: PackageIndexTrait> ExportTrait<Index> for StringTableExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;
        asset.write_i32::<LE>(0)?;

        asset.write_fstring(self.namespace.as_deref())?;
        asset.write_i32::<LE>(self.table.len() as i32)?;
        for (_, key, value) in &self.table {
            asset.write_fstring(Some(key))?;
            asset.write_fstring(Some(value))?;
        }
        Ok(())
    }
}
