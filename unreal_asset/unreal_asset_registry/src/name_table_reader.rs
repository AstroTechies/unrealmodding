//! Asset registry NameTableReader
use std::io::{self, Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use unreal_asset_base::{
    containers::{IndexedMap, NameMap, SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::EngineVersion,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::{ArchiveReader, ArchiveTrait, ArchiveType, PassthroughArchiveReader},
    types::{FName, PackageIndex},
    unversioned::Usmap,
    Error, Import,
};

/// Used for reading NameTable entries by modifying the behavior
/// of some of the value read methods.
pub struct NameTableReader<'reader, Reader: ArchiveReader> {
    /// Reader
    reader: &'reader mut Reader,
    /// Name map
    pub(crate) name_map: SharedResource<NameMap>,
}

impl<'reader, Reader: ArchiveReader> NameTableReader<'reader, Reader> {
    /// Create a new `NameTableReader` from another `Reader`
    pub(crate) fn new(reader: &'reader mut Reader) -> Result<Self, Error> {
        let name_offset = reader.read_i64::<LE>()?;
        // todo: length checking

        let mut name_map = NameMap::new();
        if name_offset > 0 {
            let original_offset = reader.position();
            reader.seek(SeekFrom::Start(name_offset as u64))?;

            let name_count = reader.read_i32::<LE>()?;
            if name_count < 0 {
                return Err(Error::invalid_file("Corrupted file".to_string()));
            }

            for i in 0..name_count {
                let name = reader.read_fstring()?.ok_or_else(|| {
                    Error::invalid_file(format!("Name table entry {i} is missing a name"))
                })?;
                name_map.get_mut().add_name_reference(name, false);

                if reader.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
                    let _non_case_preserving_hash = reader.read_u16::<LE>()?;
                    let _case_preserving_hash = reader.read_u16::<LE>()?;
                }
            }

            reader.seek(SeekFrom::Start(original_offset))?;
        }
        Ok(NameTableReader { reader, name_map })
    }
}

impl<'reader, Reader: ArchiveReader> ArchiveTrait for NameTableReader<'reader, Reader> {
    #[inline(always)]
    fn get_archive_type(&self) -> ArchiveType {
        self.reader.get_archive_type()
    }

    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.reader.get_custom_version::<T>()
    }

    fn has_unversioned_properties(&self) -> bool {
        self.reader.has_unversioned_properties()
    }

    fn use_event_driven_loader(&self) -> bool {
        self.reader.use_event_driven_loader()
    }

    fn position(&mut self) -> u64 {
        self.reader.position()
    }

    fn set_position(&mut self, pos: u64) -> io::Result<()> {
        self.reader.set_position(pos)
    }

    fn get_name_map(&self) -> SharedResource<NameMap> {
        self.name_map.clone()
    }

    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_array_struct_type_override()
    }

    fn get_map_key_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_map_key_override()
    }

    fn get_map_value_override(&self) -> &IndexedMap<String, String> {
        self.reader.get_map_value_override()
    }

    fn get_engine_version(&self) -> EngineVersion {
        self.reader.get_engine_version()
    }

    fn get_object_version(&self) -> ObjectVersion {
        self.reader.get_object_version()
    }

    fn get_object_version_ue5(&self) -> ObjectVersionUE5 {
        self.reader.get_object_version_ue5()
    }

    fn get_mappings(&self) -> Option<&Usmap> {
        self.reader.get_mappings()
    }

    fn get_parent_class_export_name(&self) -> Option<FName> {
        self.reader.get_parent_class_export_name()
    }

    fn get_import(&self, index: PackageIndex) -> Option<Import> {
        self.reader.get_import(index)
    }
}

impl<'reader, Reader: ArchiveReader> PassthroughArchiveReader for NameTableReader<'reader, Reader> {
    type Passthrough = Reader;

    #[inline(always)]
    fn get_passthrough(&mut self) -> &mut Self::Passthrough {
        self.reader
    }
}

impl<'reader, Reader: ArchiveReader> Read for NameTableReader<'reader, Reader> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<'reader, Reader: ArchiveReader> Seek for NameTableReader<'reader, Reader> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}
