//! .utoc/.ucas files support
//! Supports reading and writing.

use std::{
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
use unreal_asset_exports::Export;
use unreal_asset_proc_macro::FNameContainer;

use crate::asset_data::{AssetData, AssetTrait};
use crate::package_file_summary::PackageFileSummary;

use self::{
    exports::ExportMapEntry,
    global::IoGlobalData,
    zen::{ZenPackageSummary, ZenPackageVersioningInfo},
};

pub mod align;
pub mod cas;
pub mod encryption;
pub mod exports;
pub mod flags;
pub mod global;
pub mod providers;
pub mod toc;
pub mod zen;

/// IoStore mapped name type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EMappedNameType {
    /// Package-level name table
    Package,
    /// Container-level name table
    Container,
    /// Global name table
    Global,
}

/// IoStore mapped name
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FMappedName {
    /// Name index
    pub index: u32,
    /// Name number
    pub number: u32,
    /// Name type
    pub ty: EMappedNameType,
}

impl FMappedName {
    /// FMappedName index bits
    pub const INDEX_BITS: u32 = 30;
    /// FMappedName index mask
    pub const INDEX_MASK: u32 = (1u32 << Self::INDEX_BITS).overflowing_sub(1).0;
    /// FMappedName type mask
    pub const TYPE_MASK: u32 = !Self::INDEX_MASK;
    /// FMappedName type shift
    pub const TYPE_SHIFT: u32 = Self::INDEX_BITS;

    /// Create a new `FMappedName` instance
    pub fn new(index: u32, number: u32, ty: EMappedNameType) -> Self {
        FMappedName { index, number, ty }
    }

    /// Read `FMappedName` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let index = archive.read_u32::<LE>()?;
        let number = archive.read_u32::<LE>()?;

        let ty = EMappedNameType::try_from(((index & Self::TYPE_MASK) >> Self::TYPE_SHIFT) as u8)?;

        Ok(FMappedName {
            index: index & Self::INDEX_MASK,
            number,
            ty,
        })
    }

    /// Write `FMappedName` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        let index = self.index & Self::INDEX_MASK | (self.ty as u32) << Self::TYPE_SHIFT;

        archive.write_u32::<LE>(index)?;
        archive.write_u32::<LE>(self.number)?;

        Ok(())
    }
}

/// IoStore serialized fname entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FNameEntrySerialized {
    /// Name
    pub name: Option<String>,
}

impl FNameEntrySerialized {
    /// Create a new `FNameEntrySerialized` instance
    pub fn new(name: Option<String>) -> Self {
        FNameEntrySerialized { name }
    }

    /// Read `FNameEntrySerialized` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let name = archive.read_fstring()?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
            let _non_case_preserving_hash = archive.read_u16::<LE>()?;
            let _case_preserving_hash = archive.read_u16::<LE>()?;
        }

        Ok(FNameEntrySerialized { name })
    }

    /// Write `FNameEntrySerialized` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_fstring(self.name.as_deref())?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
            let non_case_preserving_hash = self
                .name
                .as_ref()
                .map(|e| crc::non_case_preserving_hash(e.as_str()))
                .unwrap_or(0);
            let case_preserving_hash = self
                .name
                .as_ref()
                .map(|e| crc::case_preserving_hash(e.as_str()))
                .unwrap_or(0);

            archive.write_u16::<LE>(non_case_preserving_hash)?;
            archive.write_u16::<LE>(case_preserving_hash)?;
        }

        Ok(())
    }

    /// Read an `FNameEntrySerialized` name batch from an archive
    pub fn read_name_batch<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
    ) -> Result<Vec<Self>, Error> {
        let num_strings = archive.read_i32::<LE>()?;
        if num_strings == 0 {
            return Ok(Vec::new());
        }

        let _strings_length = archive.read_u32::<LE>()?;
        let hash_version = archive.read_u64::<LE>()?;

        let _hashes = match hash_version {
            hash if hash == enums::HASH_VERSION_CITYHASH64 => {
                let mut hashes = Vec::with_capacity(num_strings as usize);
                for _ in 0..num_strings {
                    hashes.push(archive.read_u64::<LE>()?); // cityhash64 of crc::to_lower_string
                }
                Ok(hashes)
            }
            _ => Err(Error::unimplemented(format!(
                "Unimplemented name batch algorithm: {}",
                hash_version
            ))),
        }?;

        let headers = archive
            .read_array_with_length(num_strings, |reader| SerializedNameHeader::read(reader))?;

        let mut entries = Vec::with_capacity(num_strings as usize);
        for header in headers {
            entries.push(FNameEntrySerialized::new(
                archive.read_fstring_len_noterm(header.len, header.is_wide)?,
            ));
        }

        Ok(entries)
    }

    /// Read an `FNameEntrySerialized` name batch from an archive using the old method
    pub fn read_name_batch_old<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
        length: usize,
    ) -> Result<Vec<Self>, Error> {
        let mut entries = Vec::with_capacity(length);
        for _ in 0..length {
            let header = SerializedNameHeader::read(archive)?;
            let name = FNameEntrySerialized::new(
                archive.read_fstring_len_noterm(header.len, header.is_wide)?,
            );
        }

        Ok(entries)
    }
}

/// IoStore packge object index type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum EPackageObjectIndexType {
    /// Export
    Export,
    /// Script import
    ScriptImport,
    /// Package import
    PackageImport,
    /// Null
    Null,
}

/// IoStore package index
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    /// Type mask
    pub const TYPE_MASK: u64 = !Self::INDEX_MASK;
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
        let ty = EPackageObjectIndexType::try_from(
            ((type_and_id >> Self::TYPE_SHIFT) & Self::TYPE_MASK) as u16,
        )?;

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
}

impl PackageIndexTrait for PackageObjectIndex {
    fn is_import(&self) -> bool {
        false
    }

    fn is_export(&self) -> bool {
        false
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
        };

        io_asset.set_engine_version(engine_version);
        io_asset.parse_data()?;

        Ok(io_asset)
    }

    /// Set asset engine version
    fn set_engine_version(&mut self, engine_version: EngineVersion) {
        self.asset_data.set_engine_version(engine_version);
        self.raw_reader.object_version = self.asset_data.object_version;
        self.raw_reader.object_version_ue5 = self.asset_data.object_version_ue5;
    }

    /// Parse asset data
    fn parse_data(&mut self) -> Result<(), Error> {
        if self.get_engine_version() >= EngineVersion::VER_UE5_0 {
            let summary = ZenPackageSummary::read(self)?;

            let export_count = (summary.export_bundle_entries_offset - summary.export_map_offset)
                / size_of::<ExportMapEntry>() as i32;
            let import_count = (summary.export_map_offset - summary.import_map_offset)
                / size_of::<PackageObjectIndex>() as i32;

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
                self.asset_data.summary.file_licensee_version =
                    versioning_info.file_licensee_version;

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
        } else {
        }
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

    fn get_object_name(&self, _: PackageObjectIndex) -> Option<FName> {
        None
    }

    fn get_object_name_packageindex(&self, _: PackageIndex) -> Option<FName> {
        None
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
