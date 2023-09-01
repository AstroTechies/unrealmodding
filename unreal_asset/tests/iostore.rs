use std::{collections::HashMap, io::Cursor};

use unreal_asset::iostore::{
    cas::reader::IoStoreReader, container_header::IoContainerHeader, global::IoGlobalData,
    providers::memory::IoStoreMemoryProvider, toc::IoStoreTocResource, IoAsset,
};
use unreal_asset_base::{engine_version::EngineVersion, Error};

macro_rules! assets_folder {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/iostore/")
    };
}

const GLOBAL_TOC: &[u8] = include_bytes!(concat!(assets_folder!(), "global.utoc"));
const GLOBAL_CAS: &[u8] = include_bytes!(concat!(assets_folder!(), "global.ucas"));

const CONTENT_TOC: &[u8] = include_bytes!(concat!(assets_folder!(), "IoStoreTesting-Windows.utoc"));
const CONTENT_CAS: &[u8] = include_bytes!(concat!(assets_folder!(), "IoStoreTesting-Windows.ucas"));

#[test]
fn toc() -> Result<(), Error> {
    let toc_resource = IoStoreTocResource::read(&mut Cursor::new(GLOBAL_TOC), None)?;

    let mut writer = Cursor::new(Vec::new());
    toc_resource.write(&mut writer, None)?;

    assert_eq!(GLOBAL_TOC, writer.get_ref());

    Ok(())
}

#[test]
fn global_read() -> Result<(), Error> {
    let toc_resource = IoStoreTocResource::read(&mut Cursor::new(GLOBAL_TOC), None)?;

    let provider =
        IoStoreMemoryProvider::new(HashMap::from([(String::from("global.ucas"), GLOBAL_CAS)]));

    let mut reader = IoStoreReader::new(provider, "global", toc_resource, None)?;

    assert!(IoGlobalData::read(&mut reader, EngineVersion::VER_UE5_2).is_ok());

    Ok(())
}

#[test]
fn asset_parse() -> Result<(), Error> {
    let toc_resource = IoStoreTocResource::read(&mut Cursor::new(GLOBAL_TOC), None)?;
    let provider =
        IoStoreMemoryProvider::new(HashMap::from([(String::from("global.ucas"), GLOBAL_CAS)]));

    let mut reader = IoStoreReader::new(provider, "global", toc_resource, None)?;

    let global_data = IoGlobalData::read(&mut reader, EngineVersion::VER_UE5_2)?;

    let toc_resource = IoStoreTocResource::read(&mut Cursor::new(CONTENT_TOC), None)?;
    let provider = IoStoreMemoryProvider::new(HashMap::from([(
        String::from("IoStoreTesting-Windows.ucas"),
        CONTENT_CAS,
    )]));

    let mut reader = IoStoreReader::new(provider, "IoStoreTesting-Windows", toc_resource, None)?;

    let index_map = reader
        .toc_resource
        .directory_index
        .as_ref()
        .unwrap()
        .build_index_map();

    let index = index_map
        .get("/IoStoreTesting/Content/ParseMe.uasset")
        .unwrap();

    let file = reader.toc_resource.chunk_offsets_lengths[*index as usize];
    println!("{:#?}", file);

    let mut buf = vec![0u8; file.length as usize];
    reader.read_all(file.offset, &mut buf)?;

    let mut asset = IoAsset::new(
        Cursor::new(buf),
        None,
        None,
        global_data,
        EngineVersion::VER_UE5_2,
        None,
        None,
    )?;

    println!("{:#?}", asset);

    Ok(())
}
