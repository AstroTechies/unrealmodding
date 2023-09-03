//! .utoc/.ucas files support
//! Supports reading and writing.

use std::{
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
    mem::size_of,
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_base::{
    cast,
    containers::{
        chain::Chain, indexed_map::IndexedMap, name_map::NameMap, shared_resource::SharedResource,
    },
    crc,
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::{get_object_versions, EngineVersion},
    enums,
    error::Error,
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    passthrough_archive_reader,
    reader::{
        archive_reader::ArchiveReader,
        archive_trait::{ArchiveTrait, ArchiveType},
        archive_writer::ArchiveWriter,
        raw_reader::RawReader,
    },
    types::{fname::FName, PackageIndex, PackageIndexTrait, SerializedNameHeader},
    unversioned::Usmap,
    Import,
};
use unreal_asset_exports::{BaseExport, Export};
use unreal_asset_proc_macro::FNameContainer;

use crate::asset_data::{AssetData, AssetTrait, ExportReaderTrait};
use crate::package_file_summary::PackageFileSummary;

use self::{
    container_header::IoContainerHeader,
    exports::{EExportCommandType, ExportBundleEntry, ExportBundleHeader, IoStoreExportMapEntry},
    flags::EExportFilterFlags,
    global::IoGlobalData,
    graph_data::IoStoreGraphData,
    name::{EMappedNameType, FMappedName, FNameEntrySerialized},
    package_id::PackageId,
    zen::{ZenPackageSummary, ZenPackageVersioningInfo},
};

pub mod align;
pub mod cas;
pub mod container_header;
pub mod encryption;
pub mod exports;
pub mod flags;
pub mod global;
pub mod graph_data;
pub mod name;
pub mod package_id;
pub mod package_store_entry;
pub mod providers;
pub mod toc;
pub mod zen;

/// IoStore packge object index type
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum EPackageObjectIndexType {
    /// Export
    Export,
    /// Script import
    ScriptImport,
    /// Package import
    PackageImport,
    /// Null
    #[default]
    Null,
}

/// IoStore package index
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackageObjectIndex {
    /// Id
    pub id: u64,
    /// Type
    pub ty: EPackageObjectIndexType,
}

impl PackageObjectIndex {
    /// Index bits
    pub const INDEX_BITS: u64 = 62;
    /// Index mask
    pub const INDEX_MASK: u64 = (1u64 << Self::INDEX_BITS).overflowing_sub(1).0;
    /// Type bit shift
    pub const TYPE_SHIFT: u64 = Self::INDEX_BITS;

    /// Create a new `PackageObjectIndex` instance
    pub fn new(id: u64, ty: EPackageObjectIndexType) -> Self {
        PackageObjectIndex { id, ty }
    }

    /// Read `PackageObjectIndex` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let type_and_id = archive.read_u64::<LE>()?;

        let id = type_and_id & Self::INDEX_MASK;
        let ty = EPackageObjectIndexType::try_from((type_and_id >> Self::TYPE_SHIFT) as u16)?;

        Ok(PackageObjectIndex { id, ty })
    }

    /// Write `PackageObjectIndex` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        let type_and_id = self.id & Self::INDEX_MASK | ((self.ty as u64) << Self::TYPE_SHIFT);

        archive.write_u64::<LE>(type_and_id)?;

        Ok(())
    }

    /// Check if this `PackageObjectIndex` is null
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.ty == EPackageObjectIndexType::Null
    }

    /// Check if this `PackageObjectIndex` is a package import
    #[inline(always)]
    pub fn is_package_import(&self) -> bool {
        self.ty == EPackageObjectIndexType::PackageImport
    }

    /// Check if this `PackageObjectIndex` is a script import
    #[inline(always)]
    pub fn is_script_import(&self) -> bool {
        self.ty == EPackageObjectIndexType::ScriptImport
    }

    /// Get this `PackageObjectIndex` as an export
    #[inline(always)]
    pub fn as_export(&self) -> u32 {
        self.id as u32
    }

    /// Get `PackageObjectIndex` serialized size
    #[inline(always)]
    pub fn serialized_size() -> u64 {
        size_of::<u64>() as u64
    }
}

impl PackageIndexTrait for PackageObjectIndex {
    #[inline(always)]
    fn is_import(&self) -> bool {
        self.is_package_import() || self.is_script_import()
    }

    #[inline(always)]
    fn is_export(&self) -> bool {
        self.ty == EPackageObjectIndexType::Export
    }
}

impl ToString for PackageObjectIndex {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

/// IoStore script object entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScriptObjectEntry {
    /// Object name
    pub object_name: FMappedName,
    /// Global index
    pub global_index: PackageObjectIndex,
    /// Outer index
    pub outer_index: PackageObjectIndex,
    /// Class default object index
    pub cdo_index: PackageObjectIndex,
}

impl ScriptObjectEntry {
    /// Create a new `ScriptObjectEntry` instance
    pub fn new(
        object_name: FMappedName,
        global_index: PackageObjectIndex,
        outer_index: PackageObjectIndex,
        cdo_index: PackageObjectIndex,
    ) -> Self {
        ScriptObjectEntry {
            object_name,
            global_index,
            outer_index,
            cdo_index,
        }
    }

    /// Read `ScriptObjectEntry` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let object_name = FMappedName::read(archive)?;
        let global_index = PackageObjectIndex::read(archive)?;
        let outer_index = PackageObjectIndex::read(archive)?;
        let cdo_index = PackageObjectIndex::read(archive)?;

        Ok(ScriptObjectEntry {
            object_name,
            global_index,
            outer_index,
            cdo_index,
        })
    }

    /// Write `ScriptObjectEntry` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        self.object_name.write(archive)?;
        self.global_index.write(archive)?;
        self.outer_index.write(archive)?;
        self.cdo_index.write(archive)?;

        Ok(())
    }
}

/// IoStore bulk data map entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BulkDataMapEntry {
    /// Serialized offset
    pub serial_offset: u64,
    /// Duplicate serialized offset
    pub dup_serial_offset: u64,
    /// Serialized size
    pub serial_size: u64,
    /// Flags
    pub flags: u32,
    /// Padding
    pub padding: u32,
}

impl BulkDataMapEntry {
    /// Read `BulkDataMapEntry` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let serial_offset = archive.read_u64::<LE>()?;
        let dup_serial_offset = archive.read_u64::<LE>()?;
        let serial_size = archive.read_u64::<LE>()?;
        let flags = archive.read_u32::<LE>()?;
        let padding = archive.read_u32::<LE>()?;

        Ok(BulkDataMapEntry {
            serial_offset,
            dup_serial_offset,
            serial_size,
            flags,
            padding,
        })
    }

    /// Write `BulkDataMapEntry` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_u64::<LE>(self.serial_offset)?;
        archive.write_u64::<LE>(self.dup_serial_offset)?;
        archive.write_u64::<LE>(self.serial_size)?;
        archive.write_u32::<LE>(self.flags)?;
        archive.write_u32::<LE>(self.padding)?;

        Ok(())
    }
}

/// IoStore asset
#[derive(FNameContainer)]
pub struct IoAsset<C: Read + Seek> {
    /// Raw reader
    #[container_ignore]
    pub raw_reader: RawReader<PackageObjectIndex, C>,
    /// Asset data
    pub asset_data: AssetData<PackageObjectIndex>,
    /// Asset global data
    #[container_ignore]
    global_data: SharedResource<IoGlobalData>,
    /// Name map
    #[container_ignore]
    name_map: SharedResource<NameMap>,
    /// Asset name
    pub name: FName,
    /// Export map
    #[container_ignore]
    pub export_map: Vec<IoStoreExportMapEntry>,
    /// Graph data
    #[container_ignore]
    pub graph_data: Option<IoStoreGraphData>,
}

impl<C: Read + Seek> IoAsset<C> {
    /// Create `IoAsset` from a binary file
    pub fn new(
        asset_data: C,
        bulk_data: Option<C>,
        ptnl_data: Option<C>,
        global_data: SharedResource<IoGlobalData>,
        engine_version: EngineVersion,
        mappings: Option<Usmap>,
        container_header: Option<&IoContainerHeader>,
    ) -> Result<Self, Error> {
        let use_event_driven_loader = bulk_data.is_some();

        let chain = Chain::new(asset_data, bulk_data);
        // todo: uptnl

        let (object_version, object_version_ue5) = get_object_versions(engine_version);

        let name_map = NameMap::new();
        let raw_reader = RawReader::new(
            chain,
            object_version,
            object_version_ue5,
            use_event_driven_loader,
            name_map.clone(),
        );

        let mut io_asset = IoAsset {
            raw_reader,
            asset_data: AssetData {
                use_event_driven_loader,
                mappings,
                ..Default::default()
            },
            global_data,
            name_map,
            name: FName::default(),
            export_map: Vec::default(),
            graph_data: None,
        };

        io_asset.set_engine_version(engine_version);
        io_asset.parse_data(container_header)?;

        Ok(io_asset)
    }

    /// Set asset engine version
    fn set_engine_version(&mut self, engine_version: EngineVersion) {
        self.asset_data.set_engine_version(engine_version);
        self.raw_reader.object_version = self.asset_data.object_version;
        self.raw_reader.object_version_ue5 = self.asset_data.object_version_ue5;
    }

    /// Parse asset data
    fn parse_data(&mut self, container_header: Option<&IoContainerHeader>) -> Result<(), Error> {
        // todo < 5.0 support
        let summary = ZenPackageSummary::read(self)?;

        let export_count = (summary.export_bundle_entries_offset - summary.export_map_offset)
            / IoStoreExportMapEntry::serialized_size() as i32;
        let import_count = (summary.export_map_offset - summary.import_map_offset)
            / PackageObjectIndex::serialized_size() as i32;

        let package_summary = PackageFileSummary {
            package_flags: summary.package_flags,
            export_count,
            import_count,
            file_licensee_version: 0,
            custom_versions: Vec::new(),
            unversioned: true,
        };
        self.asset_data.summary = package_summary;

        if summary.has_versioning_info {
            let versioning_info = ZenPackageVersioningInfo::read(self)?;
            self.asset_data.summary.file_licensee_version = versioning_info.file_licensee_version;

            self.asset_data.object_version = versioning_info.object_version;
            self.asset_data.object_version_ue5 = versioning_info.object_version_ue5;

            self.raw_reader.object_version = self.asset_data.object_version;
            self.raw_reader.object_version_ue5 = self.asset_data.object_version_ue5;

            self.asset_data.summary.custom_versions = versioning_info.custom_versions;

            self.asset_data.summary.unversioned = false;
        }

        self.name_map = NameMap::from_name_batch(
            &FNameEntrySerialized::read_name_batch(self)?
                .into_iter()
                .filter_map(|e| e.name)
                .collect::<Vec<_>>(),
        );

        self.name = self.mapped_name_to_fname(summary.name);

        let store_entry = match container_header {
            Some(container_header) => {
                let package_id = self.name.get_content(PackageId::from_name);

                container_header
                    .main_segment
                    .package_ids
                    .iter()
                    .position(|e| *e == package_id)
                    .map(|e| container_header.main_segment.entries[e].clone())
                    .or_else(|| {
                        container_header.optional_segment.as_ref().and_then(|e| {
                            e.package_ids
                                .iter()
                                .position(|i| *i == package_id)
                                .map(|i| e.entries[i].clone())
                        })
                    })
            }
            None => None,
        };

        let bulk_data_map = match self.get_object_version_ue5() >= ObjectVersionUE5::DATA_RESOURCES
        {
            true => {
                let size = self.read_u64::<LE>()?;
                let count = (size as usize / size_of::<BulkDataMapEntry>()) as i32;
                Some(self.read_array_with_length(count, BulkDataMapEntry::read)?)
            }
            false => None,
        };

        // imported public export hashes
        self.set_position(summary.imported_public_export_hashes_offset as u64)?;

        let imported_public_export_hashes = self.read_array_with_length(
            (summary.import_map_offset - summary.imported_public_export_hashes_offset)
                / size_of::<u64>() as i32,
            |reader| Ok(reader.read_u64::<LE>()?),
        )?;

        // import map
        self.set_position(summary.import_map_offset as u64)?;

        let import_map = self.read_array_with_length(
            self.asset_data.summary.import_count,
            PackageObjectIndex::read,
        )?;

        // export map
        self.set_position(summary.export_map_offset as u64)?;

        self.export_map = self.read_array_with_length(
            self.asset_data.summary.export_count,
            IoStoreExportMapEntry::read,
        )?;

        // export bundle entries
        self.set_position(summary.export_bundle_entries_offset as u64)?;

        let export_bundle_entries = self.read_array_with_length(
            self.asset_data.summary.export_count * 2,
            ExportBundleEntry::read,
        )?;

        let imported_package_ids = store_entry
            .as_ref()
            .map(|e| e.imported_packages.clone())
            .unwrap_or_default();

        let graph_data = match summary.graph_data_offset {
            Some(graph_data_offset) => {
                // export bundle headers

                self.set_position(graph_data_offset as u64)?;

                let export_bundle_headers_count = store_entry
                    .as_ref()
                    .and_then(|e| e.export_bundle_count)
                    .unwrap_or(1);

                Some(IoStoreGraphData::read(
                    self,
                    export_bundle_headers_count,
                    &imported_package_ids,
                )?)
            }
            None => None,
        };

        // todo: attach ubulk/uptnl

        match graph_data {
            Some(ref graph_data) => {
                for bundle in &graph_data.export_bundle_headers {
                    let mut offset = summary.header_size as i64;

                    for i in 0..bundle.entry_count {
                        let entry = export_bundle_entries[(bundle.first_entry_index + i) as usize];

                        if entry.command_type != EExportCommandType::Serialize {
                            continue;
                        }

                        let export_map_entry =
                            self.export_map[entry.local_export_index as usize].clone();
                        let mut base_export =
                            self.export_map_entry_to_base_export(export_map_entry)?;

                        base_export.serial_offset = offset;
                        let serial_size = base_export.serial_size;
                        let next_starting = base_export.serial_offset + serial_size;

                        let export = self.read_export(base_export, next_starting as u64)?;

                        offset += serial_size;

                        self.asset_data.exports.push(export);
                    }
                }
            }
            None => {
                unimplemented!()
            }
        };

        self.graph_data = graph_data;

        Ok(())
    }

    /// Create an [`FName`] from an [`FMappedName`]
    fn mapped_name_to_fname(&mut self, mapped_name: FMappedName) -> FName {
        match mapped_name.ty == EMappedNameType::Global {
            true => self
                .global_data
                .get_mut()
                .global_name_map
                .get_mut()
                .create_fname(mapped_name.index as i32, mapped_name.number as i32),
            false => self
                .name_map
                .get_mut()
                .create_fname(mapped_name.index as i32, mapped_name.number as i32),
        }
    }

    /// Get [`ScriptObjectEntry`] from a [`PackageObjectIndex`]
    pub fn get_script_import(&self, index: PackageObjectIndex) -> Option<ScriptObjectEntry> {
        if !index.is_script_import() {
            return None;
        }

        self.global_data
            .get_ref()
            .script_object_entries
            .iter()
            .find(|e| e.global_index == index)
            .copied()
    }

    /// Get [`IoStoreExportMapEntry`] from a [`PackageObjectIndex`]
    pub fn get_export_map_entry(
        &self,
        index: PackageObjectIndex,
    ) -> Option<&IoStoreExportMapEntry> {
        if !index.is_export() {
            return None;
        }

        Some(&self.export_map[index.as_export() as usize])
    }

    /// Convert [`ExportMapEntry`] to [`BaseExport`]
    pub fn export_map_entry_to_base_export(
        &mut self,
        entry: IoStoreExportMapEntry,
    ) -> Result<BaseExport<PackageObjectIndex>, Error> {
        let mut export = BaseExport::<PackageObjectIndex>::default();

        export.serial_offset = entry.cooked_serial_offset as i64;
        export.serial_size = entry.cooked_serial_size as i64;

        export.object_name = self.mapped_name_to_fname(entry.object_name);
        export.outer_index = entry.outer_index;
        export.class_index = entry.class_index;
        export.super_index = entry.super_index;
        export.template_index = entry.template_index;

        // todo: pre 5.0 support
        export.public_export_hash = entry
            .public_export_hash
            .ok_or_else(|| Error::invalid_file("Pre 5.0 IoStore is not supported".to_string()))?;

        export.object_flags = entry.object_flags;

        export.not_for_client = entry
            .filter_flags
            .contains(EExportFilterFlags::NOT_FOR_CLIENT);
        export.not_for_server = entry
            .filter_flags
            .contains(EExportFilterFlags::NOT_FOR_SERVER);

        Ok(export)
    }
}

impl<C: Read + Seek> AssetTrait<PackageObjectIndex> for IoAsset<C> {
    fn get_asset_data(&self) -> &AssetData<PackageObjectIndex> {
        &self.asset_data
    }

    fn get_asset_data_mut(&mut self) -> &mut AssetData<PackageObjectIndex> {
        &mut self.asset_data
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn search_name_reference(&self, name: &str) -> Option<i32> {
        self.name_map.get_ref().search_name_reference(name)
    }

    fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
        self.name_map
            .get_mut()
            .add_name_reference(name, force_add_duplicates)
    }

    fn get_name_reference<T>(&self, index: i32, func: impl FnOnce(&str) -> T) -> T {
        func(self.name_map.get_ref().get_name_reference(index))
    }

    fn add_fname(&mut self, slice: &str) -> FName {
        self.name_map.get_mut().add_fname(slice)
    }
}

impl<C: Read + Seek> ArchiveTrait<PackageObjectIndex> for IoAsset<C> {
    fn get_archive_type(&self) -> ArchiveType {
        ArchiveType::Zen
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.asset_data.get_custom_version::<T>()
    }

    fn has_unversioned_properties(&self) -> bool {
        self.asset_data.has_unversioned_properties()
    }

    fn use_event_driven_loader(&self) -> bool {
        self.asset_data.use_event_driven_loader
    }

    fn position(&mut self) -> u64 {
        self.raw_reader.position()
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.array_struct_type_override
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.map_key_override
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        &self.asset_data.map_value_override
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.asset_data.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.asset_data.object_version
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.asset_data.object_version_ue5
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        self.asset_data.mappings.as_ref()
    }

    fn get_parent_class_export_name(&self) -> Option<FName> {
        None
    }

    fn get_object_name(&mut self, index: PackageObjectIndex) -> Option<FName> {
        self.get_script_import(index)
            .map(|e| self.mapped_name_to_fname(e.object_name))
    }

    fn get_object_name_packageindex(&self, _: PackageIndex) -> Option<FName> {
        None
    }

    fn get_export_class_type(&mut self, index: PackageObjectIndex) -> Option<FName> {
        if let Some(script_import) = self.get_script_import(index) {
            return Some(self.mapped_name_to_fname(script_import.object_name));
        }

        if let Some(export) = self.get_export_map_entry(index) {
            return Some(self.mapped_name_to_fname(export.object_name));
        }

        // todo: package import implementation
        unimplemented!();
    }
}

impl<C: Read + Seek> ArchiveReader<PackageObjectIndex> for IoAsset<C> {
    passthrough_archive_reader!(raw_reader);
}

impl<C: Read + Seek> Read for IoAsset<C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.raw_reader.read(buf)
    }
}

impl<C: Read + Seek> Seek for IoAsset<C> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.raw_reader.seek(pos)
    }
}

impl<C: Read + Seek> Debug for IoAsset<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IoAsset")
            .field("asset_data", &self.asset_data)
            .field("global_data", &self.global_data)
            .field("name", &self.name)
            .field("graph_data", &self.graph_data)
            .finish()
    }
}
