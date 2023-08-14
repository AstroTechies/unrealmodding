//! IoStore global data serialization

use std::io::{Cursor, Read, Seek};

use unreal_asset_base::containers::chain::Chain;
use unreal_asset_base::engine_version::{get_object_versions, EngineVersion};
use unreal_asset_base::error::IoStoreError;
use unreal_asset_base::reader::archive_reader::ArchiveReader;
use unreal_asset_base::reader::raw_reader::RawReader;

use crate::containers::{name_map::NameMap, shared_resource::SharedResource};
use crate::error::Error;

use super::cas::reader::IoStoreReader;
use super::providers::IoStoreProvider;
use super::toc::chunk::EIoChunkType5;
use super::{FNameEntrySerialized, PackageObjectIndex, ScriptObjectEntry};

/// IoStore global data
#[derive(Debug, Clone, PartialEq)]
pub struct IoGlobalData {
    /// Engine version
    pub engine_version: EngineVersion,
    /// Global name map
    pub global_name_map: SharedResource<NameMap>,
    /// Script object entries
    pub script_object_entries: Vec<ScriptObjectEntry>,
}

impl IoGlobalData {
    /// Read `IoGlobalData` from an [`IoStoreReader`]
    pub fn read<R: Read + Seek, P: IoStoreProvider<R>>(
        reader: &mut IoStoreReader<R, P>,
        engine_version: EngineVersion,
    ) -> Result<Self, Error> {
        let (object_version, object_version_ue5) = get_object_versions(engine_version);

        let (name_map, mut meta_reader) = match engine_version >= EngineVersion::VER_UE5_0 {
            true => {
                let chunk_info = reader
                    .toc_resource
                    .get_chunk_offset_by_type(EIoChunkType5::ScriptObjects as u8)?
                    .ok_or_else(|| IoStoreError::no_chunk("ScriptObjects"))?;
                let mut data = vec![0u8; chunk_info.length as usize];
                reader.read_all(chunk_info.offset, &mut data)?;

                let mut meta_reader = RawReader::<PackageObjectIndex, _>::new(
                    Chain::new(Cursor::new(data), None),
                    object_version,
                    object_version_ue5,
                    false,
                    NameMap::new(),
                );

                let name_batch = FNameEntrySerialized::read_name_batch(&mut meta_reader)?
                    .into_iter()
                    .filter_map(|e| e.name)
                    .collect::<Vec<_>>();
                (NameMap::from_name_batch(&name_batch), meta_reader)
            }
            false => unimplemented!(),
        };

        let script_object_entries =
            meta_reader.read_array(|reader| ScriptObjectEntry::read(reader))?;

        Ok(IoGlobalData {
            engine_version,
            global_name_map: name_map,
            script_object_entries,
        })
    }
}
