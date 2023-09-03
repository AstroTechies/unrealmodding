//! IoStore graph data

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use unreal_asset_base::{
    containers::IndexedMap,
    import,
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error,
};
use unreal_helpers::UnrealWriteExt;

use super::{
    exports::{EExportCommandType, ExportBundleHeader},
    package_id::PackageId,
};

/// IoStore internal graph arc
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FInternalArc {
    /// From (export bundle index)
    pub from: i32,
    /// To (export bundle index)
    pub to: i32,
}

impl FInternalArc {
    /// Read `FInternalArc` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let from = archive.read_i32::<LE>()?;
        let to = archive.read_i32::<LE>()?;

        Ok(FInternalArc { from, to })
    }

    /// Write `FInternalArc` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_i32::<LE>(self.from)?;
        archive.write_i32::<LE>(self.to)?;

        Ok(())
    }
}

/// IoStore external graph arc
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FExternalArc {
    /// From (import index)
    pub from: i32,
    /// From command type
    pub from_command_type: EExportCommandType,
    /// To (export bundle index)
    pub to: i32,
}

impl FExternalArc {
    /// Read `FExternalArc` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let from = archive.read_i32::<LE>()?;
        let from_command_type = EExportCommandType::try_from(archive.read_u8()? as u32)?;
        let to = archive.read_i32::<LE>()?;

        Ok(FExternalArc {
            from,
            from_command_type,
            to,
        })
    }

    /// Write `FExternalArc` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_i32::<LE>(self.from)?;
        archive.write_u8(self.from_command_type as u8)?;
        archive.write_i32::<LE>(self.to)?;

        Ok(())
    }
}

/// IoStore graph data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoStoreGraphData {
    /// Export bundle headers
    pub export_bundle_headers: Vec<ExportBundleHeader>,
    /// Internal arcs
    pub internal_arcs: Vec<FInternalArc>,
    /// External arcs (from imported packages)
    pub arcs_from_imported_packages: IndexedMap<PackageId, Vec<FExternalArc>>,
}

impl IoStoreGraphData {
    /// Read `IoStoreGraphData` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
        export_bundle_headers_count: i32,
        imported_package_ids: &[PackageId],
    ) -> Result<Self, Error> {
        let export_bundle_headers = archive
            .read_array_with_length(export_bundle_headers_count, ExportBundleHeader::read)?;

        let internal_arcs = archive.read_array(FInternalArc::read)?;

        let mut arcs_from_imported_packages = IndexedMap::new();

        for imported_package_id in imported_package_ids {
            let external_arcs = archive.read_array(FExternalArc::read)?;
            if !external_arcs.is_empty() {
                arcs_from_imported_packages.insert(*imported_package_id, external_arcs);
            }
        }

        Ok(IoStoreGraphData {
            export_bundle_headers,
            internal_arcs,
            arcs_from_imported_packages,
        })
    }

    /// Write `IoStoreGraphData` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
        imported_package_ids: &[PackageId],
    ) -> Result<(), Error> {
        for export_bundle_header in &self.export_bundle_headers {
            export_bundle_header.write(archive)?;
        }

        let mut internal_arcs = self.internal_arcs.clone();
        internal_arcs.sort_by(|a, b| {
            if a.to == b.to {
                return a.from.cmp(&b.from);
            }
            return a.to.cmp(&b.to);
        });

        archive.write_array(&internal_arcs, |writer, e| e.write(writer))?;

        for imported_package_id in imported_package_ids {
            match self
                .arcs_from_imported_packages
                .get_by_key(imported_package_id)
            {
                Some(e) => {
                    let mut sorted = e.clone();
                    sorted.sort_by(|a, b| {
                        let to = a.to.cmp(&b.to);
                        let from = a.from.cmp(&b.from);
                        let from_command_type = a.from_command_type.cmp(&b.from_command_type);

                        match (to.is_eq(), from.is_eq()) {
                            (true, true) => from_command_type,
                            (true, false) => from,
                            _ => to,
                        }
                    });

                    archive.write_array(&sorted, |writer, e| e.write(writer))?;
                }
                None => {
                    archive.write_i32::<LE>(0)?;
                }
            }
        }

        Ok(())
    }
}
