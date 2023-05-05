//! Base uasset export

use std::io::Cursor;

use byteorder::LE;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::exports::{ExportBaseTrait, ExportNormalTrait, ExportTrait};
use crate::flags::EObjectFlags;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::reader::archive_reader::ArchiveReader;
use crate::reader::archive_trait::{ArchiveTrait, ArchiveType};
use crate::reader::archive_writer::ArchiveWriter;
use crate::reader::raw_writer::RawWriter;
use crate::types::{fname::FName, Guid, PackageIndex};

/// Export filter flags
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EExportFilterFlags {
    /// None
    None,
    /// This export should not be loaded on the client
    NotForClient,
    /// This export should not be loaded on the server
    NotForServer,
}

/// Minimal information about an export
#[derive(FNameContainer, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BaseExport {
    /// Class index
    #[container_ignore]
    pub class_index: PackageIndex,
    /// Zen class index

    /// Super index
    #[container_ignore]
    pub super_index: PackageIndex,
    /// Zen super index

    /// Template index
    #[container_ignore]
    pub template_index: PackageIndex,
    /// Zen template index

    /// Outer index
    #[container_ignore]
    pub outer_index: PackageIndex,
    /// Zen outer index

    /// Object name
    pub object_name: FName,
    /// Object flags
    #[container_ignore]
    pub object_flags: EObjectFlags,
    /// Serialized size
    pub serial_size: i64,
    /// Serialized offset
    pub serial_offset: i64,
    /// Is forced export
    pub forced_export: bool,
    /// Is not for client
    pub not_for_client: bool,
    /// Is not for server
    pub not_for_server: bool,
    /// Package guid
    pub package_guid: Guid,
    /// Is inherited instance
    pub is_inherited_instance: bool,
    /// Package flags
    pub package_flags: u32,
    /// Is not always loaded for editor game
    pub not_always_loaded_for_editor_game: bool,
    /// Is an asset
    pub is_asset: bool,
    /// Generate public hash
    pub generate_public_hash: bool,
    /// Public export hash. Interpreted as a global import PackageObjectIndex in UE4 assets
    pub public_export_hash: u64,
    /// First dependency serialized offset
    pub first_export_dependency_offset: i32,
    /// Dependencies that should be serialized before this export is serialized
    #[container_ignore]
    pub serialization_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_serialization_dependencies_size: i32,

    /// Dependencies that should be created before this export is serialized
    #[container_ignore]
    pub create_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_serialization_dependencies_size: i32,

    /// Dependencies that should be serialized before this export is created
    #[container_ignore]
    pub serialization_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_create_dependencies_size: i32,

    /// Dependencies that should be created before this export is created
    #[container_ignore]
    pub create_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_create_dependencies_size: i32,

    /// Padding
    #[container_ignore]
    pub padding: [u8; 3],
}

impl BaseExport {
    /// Gets class type for first ancestry parent
    pub fn get_class_type_for_ancestry<Asset: ArchiveTrait>(&self, asset: &Asset) -> FName {
        match self.class_index.is_import() {
            true => asset
                .get_import(self.class_index)
                .map(|e| e.object_name.clone()),
            false => asset.get_parent_class().map(|e| e.parent_class_export_name),
        }
        .unwrap_or_default()
    }

    /// Read `BaseExport` from an archive
    pub fn read_export_map_entry<Archive: ArchiveReader>(
        reader: &mut Archive,
    ) -> Result<Self, Error> {
        match reader.get_archive_type() {
            ArchiveType::UAsset => Self::read_export_map_entry_uasset(reader),
            _ => Err(Error::archive_type_mismatch(
                &[ArchiveType::UAsset, ArchiveType::Zen],
                reader.get_archive_type(),
            )),
        }
    }

    /// UAsset specific export map entry reading implementation
    fn read_export_map_entry_uasset<Archive: ArchiveReader>(
        reader: &mut Archive,
    ) -> Result<Self, Error> {
        let mut export = BaseExport {
            class_index: PackageIndex::new(reader.read_i32::<LE>()?),
            super_index: PackageIndex::new(reader.read_i32::<LE>()?),
            ..Default::default()
        };

        if reader.get_object_version() >= ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS {
            export.template_index = PackageIndex::new(reader.read_i32::<LE>()?);
        }

        export.outer_index = PackageIndex::new(reader.read_i32::<LE>()?);
        export.object_name = reader.read_fname()?;
        export.object_flags = EObjectFlags::from_bits(reader.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid property flags".to_string()))?;

        if reader.get_object_version() < ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
            export.serial_size = reader.read_i32::<LE>()? as i64;
            export.serial_offset = reader.read_i32::<LE>()? as i64;
        } else {
            export.serial_size = reader.read_i64::<LE>()?;
            export.serial_offset = reader.read_i64::<LE>()?;
        }

        export.forced_export = reader.read_i32::<LE>()? == 1;
        export.not_for_client = reader.read_i32::<LE>()? == 1;
        export.not_for_server = reader.read_i32::<LE>()? == 1;
        reader.read_exact(&mut export.package_guid)?;

        if reader.get_object_version_ue5() >= ObjectVersionUE5::TRACK_OBJECT_EXPORT_IS_INHERITED {
            export.is_inherited_instance = reader.read_i32::<LE>()? == 1;
        }

        export.package_flags = reader.read_u32::<LE>()?;

        if reader.get_object_version() >= ObjectVersion::VER_UE4_LOAD_FOR_EDITOR_GAME {
            export.not_always_loaded_for_editor_game = reader.read_i32::<LE>()? == 1;
        }

        if reader.get_object_version() >= ObjectVersion::VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
            export.is_asset = reader.read_i32::<LE>()? == 1;
        }

        if reader.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
            export.generate_public_hash = reader.read_i32::<LE>()? == 1;
        }

        if reader.get_object_version()
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            export.first_export_dependency_offset = reader.read_i32::<LE>()?;
            export.serialization_before_serialization_dependencies_size =
                reader.read_i32::<LE>()?;
            export.create_before_serialization_dependencies_size = reader.read_i32::<LE>()?;
            export.serialization_before_create_dependencies_size = reader.read_i32::<LE>()?;
            export.create_before_create_dependencies_size = reader.read_i32::<LE>()?;
        }

        Ok(export)
    }

    /// Write `BaseExport` export map entry to an archive
    pub fn write_export_map_entry<Archive: ArchiveWriter>(
        &self,
        writer: &mut Archive,
        serial_size: i64,
        serial_offset: i64,
        first_export_dependency_offset: i32,
    ) -> Result<(), Error> {
        match writer.get_archive_type() {
            ArchiveType::UAsset => self.write_export_map_entry_uasset(
                writer,
                serial_size,
                serial_offset,
                first_export_dependency_offset,
            ),
            _ => Err(Error::archive_type_mismatch(
                &[ArchiveType::UAsset, ArchiveType::Zen],
                writer.get_archive_type(),
            )),
        }
    }

    /// UAsset specific export map entry writing implementation
    fn write_export_map_entry_uasset<Archive: ArchiveWriter>(
        &self,
        writer: &mut Archive,
        serial_size: i64,
        serial_offset: i64,
        first_export_dependency_offset: i32,
    ) -> Result<(), Error> {
        writer.write_i32::<LE>(self.class_index.index)?;
        writer.write_i32::<LE>(self.super_index.index)?;

        if writer.get_object_version() >= ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS {
            writer.write_i32::<LE>(self.template_index.index)?;
        }

        writer.write_i32::<LE>(self.outer_index.index)?;
        writer.write_fname(&self.object_name)?;
        writer.write_u32::<LE>(self.object_flags.bits())?;

        if writer.get_object_version() < ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES {
            writer.write_i32::<LE>(serial_size as i32)?;
            writer.write_i32::<LE>(serial_offset as i32)?;
        } else {
            writer.write_i64::<LE>(serial_size)?;
            writer.write_i64::<LE>(serial_offset)?;
        }

        writer.write_i32::<LE>(match self.forced_export {
            true => 1,
            false => 0,
        })?;
        writer.write_i32::<LE>(match self.not_for_client {
            true => 1,
            false => 0,
        })?;
        writer.write_i32::<LE>(match self.not_for_server {
            true => 1,
            false => 0,
        })?;
        writer.write_all(&self.package_guid)?;

        if writer.get_object_version_ue5() >= ObjectVersionUE5::TRACK_OBJECT_EXPORT_IS_INHERITED {
            writer.write_i32::<LE>(match self.is_inherited_instance {
                true => 1,
                false => 0,
            })?;
        }

        writer.write_u32::<LE>(self.package_flags)?;

        if writer.get_object_version() >= ObjectVersion::VER_UE4_LOAD_FOR_EDITOR_GAME {
            writer.write_i32::<LE>(match self.not_always_loaded_for_editor_game {
                true => 1,
                false => 0,
            })?;
        }

        if writer.get_object_version() >= ObjectVersion::VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT {
            writer.write_i32::<LE>(match self.is_asset {
                true => 1,
                false => 0,
            })?;
        }

        if writer.get_object_version_ue5() >= ObjectVersionUE5::OPTIONAL_RESOURCES {
            writer.write_i32::<LE>(match self.generate_public_hash {
                true => 1,
                false => 0,
            })?;
        }

        if writer.get_object_version()
            >= ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS
        {
            writer.write_i32::<LE>(first_export_dependency_offset)?;
            writer.write_i32::<LE>(
                self.serialization_before_serialization_dependencies.len() as i32
            )?;
            writer.write_i32::<LE>(self.create_before_serialization_dependencies.len() as i32)?;
            writer.write_i32::<LE>(self.serialization_before_create_dependencies.len() as i32)?;
            writer.write_i32::<LE>(self.create_before_create_dependencies.len() as i32)?;
        }
        Ok(())
    }

    /// Get `BaseExport` export map entry size for an archive
    pub fn get_export_map_entry_size<Archive: ArchiveTrait>(
        archive: &Archive,
    ) -> Result<u64, Error> {
        let mut cursor = Cursor::new(Vec::new());
        let mut raw_writer = RawWriter::new(
            &mut cursor,
            archive.get_object_version(),
            archive.get_object_version_ue5(),
            archive.use_event_driven_loader(),
            archive.get_name_map(),
        );

        let mut default_export = BaseExport::default();
        default_export.object_name = FName::new(0, 0, archive.get_name_map());

        match archive.get_archive_type() {
            ArchiveType::UAsset => {
                default_export.write_export_map_entry_uasset(&mut raw_writer, 0, 0, 0)
            }
            _ => Err(Error::archive_type_mismatch(
                &[ArchiveType::UAsset, ArchiveType::Zen],
                archive.get_archive_type(),
            )),
        }?;

        Ok(raw_writer.position())
    }
}

impl ExportNormalTrait for BaseExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportBaseTrait for BaseExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        self
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        self
    }
}

impl ExportTrait for BaseExport {
    fn write<Writer: ArchiveWriter>(&self, _asset: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
