use std::io::SeekFrom;

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::Error;
use crate::reader::asset_reader::AssetReader;
use crate::types::FName;

#[repr(u32)]
#[derive(IntoPrimitive, TryFromPrimitive)]
pub enum EValueType {
    AnsiString,
    WideString,
    NumberlessName,
    Name,
    NumberlessExportPath,
    ExportPath,
    LocalizedText,
}

pub struct ValueId {
    pub value_type: EValueType,
    pub index: i32,
}

const TYPE_BITS: u32 = 3;
const INDEX_BITS: u32 = 32 - TYPE_BITS;

impl ValueId {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let id = asset.read_u32::<LittleEndian>()?;
        let value_type = EValueType::try_from((id << INDEX_BITS) >> INDEX_BITS)?;
        let index = id as i32 >> TYPE_BITS;

        Ok(Self { value_type, index })
    }
}

pub struct NumberedPair {
    pub key: FName,
    pub value: ValueId,
}

impl NumberedPair {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let key = asset.read_fname()?;
        let value = ValueId::new(asset)?;

        Ok(Self { key, value })
    }
}

pub struct NumberlessPair {
    pub key: u32,
    pub value: ValueId,
}

impl NumberlessPair {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let key = asset.read_u32::<LittleEndian>()?;
        let value = ValueId::new(asset)?;

        Ok(Self { key, value })
    }
}

pub struct NumberlessExportPath {
    pub class: u32,
    pub object: u32,
    pub package: u32,
}

impl NumberlessExportPath {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let class = asset.read_u32::<LittleEndian>()?;
        let object = asset.read_u32::<LittleEndian>()?;
        let package = asset.read_u32::<LittleEndian>()?;

        Ok(Self {
            class,
            object,
            package,
        })
    }
}

pub struct AssetRegistryExportPath {
    pub class: FName,
    pub object: FName,
    pub package: FName,
}

impl AssetRegistryExportPath {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let class = asset.read_fname()?;
        let object = asset.read_fname()?;
        let package = asset.read_fname()?;

        Ok(Self {
            class,
            object,
            package,
        })
    }
}

#[derive(PartialEq, Eq)]
enum ELoadOrder {
    Member,
    TextFirst,
}

pub struct Store {
    pub pairs: Vec<NumberedPair>,
    pub numberless_pairs: Vec<NumberlessPair>,
    pub ansi_strings: Vec<String>,
    pub wide_strings: Vec<String>,
    pub numberless_names: Vec<u32>,
    pub names: Vec<FName>,
    pub numberless_export_paths: Vec<NumberlessExportPath>,
    pub export_paths: Vec<AssetRegistryExportPath>,
    pub texts: Vec<Option<String>>,
}

const OLD_BEGIN_MAGIC: u32 = 0x12345678;
const BEGIN_MAGIC: u32 = 0x12345679;

impl Store {
    fn get_load_order(magic: u32) -> Result<ELoadOrder, Error> {
        match magic {
            OLD_BEGIN_MAGIC => Ok(ELoadOrder::Member),
            BEGIN_MAGIC => Ok(ELoadOrder::TextFirst),
            _ => Err(Error::invalid_file("Invalid asset store magic".to_string())),
        }
    }

    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let magic = asset.read_u32::<LittleEndian>()?;
        let order = Store::get_load_order(magic)?;

        let numberless_names_count: i32 = asset.read_i32::<LittleEndian>()?;
        let names_count: i32 = asset.read_i32::<LittleEndian>()?;
        let numberless_export_paths_count: i32 = asset.read_i32::<LittleEndian>()?;
        let export_paths_count: i32 = asset.read_i32::<LittleEndian>()?;
        let texts_count: i32 = asset.read_i32::<LittleEndian>()?;
        let ansi_string_offsets_count: i32 = asset.read_i32::<LittleEndian>()?;
        let wide_string_offsets_count: i32 = asset.read_i32::<LittleEndian>()?;
        let ansi_strings_size: i32 = asset.read_i32::<LittleEndian>()?;
        let wide_strings_size: i32 = asset.read_i32::<LittleEndian>()?;
        let numberless_pairs_count: i32 = asset.read_i32::<LittleEndian>()?;
        let pairs_count: i32 = asset.read_i32::<LittleEndian>()?;

        let mut texts = Vec::new();
        if order == ELoadOrder::TextFirst {
            asset.seek(std::io::SeekFrom::Current(4))?;
            texts = asset
                .read_array_with_length(texts_count, |asset: &mut Reader| asset.read_fstring())?;
        }

        let numberless_names = asset
            .read_array_with_length(numberless_names_count, |asset: &mut Reader| {
                Ok(asset.read_u32::<LittleEndian>()?)
            })?;

        let names =
            asset.read_array_with_length(names_count, |asset: &mut Reader| asset.read_fname())?;

        let numberless_export_paths = asset
            .read_array_with_length(numberless_export_paths_count, |asset: &mut Reader| {
                NumberlessExportPath::new(asset)
            })?;

        let export_paths = asset
            .read_array_with_length(export_paths_count, |asset: &mut Reader| {
                AssetRegistryExportPath::new(asset)
            })?;

        if order == ELoadOrder::Member {
            texts = asset
                .read_array_with_length(texts_count, |asset: &mut Reader| asset.read_fstring())?;
        }

        let ansi_string_offsets = asset
            .read_array_with_length(ansi_string_offsets_count, |asset: &mut Reader| {
                Ok(asset.read_i32::<LittleEndian>()?)
            })?;

        let wide_string_offsets = asset
            .read_array_with_length(wide_string_offsets_count, |asset: &mut Reader| {
                Ok(asset.read_i32::<LittleEndian>()?)
            })?;

        let mut ansi_strings_buf = vec![0u8; ansi_strings_size as usize];
        asset.read_exact(&mut ansi_strings_buf)?;

        let mut ansi_strings = Vec::new();

        for ansi_string_offset in ansi_string_offsets {
            let mut length = 0;
            while ansi_strings_buf
                .get((ansi_string_offset + length) as usize)
                .map(|e| *e != 0)
                .unwrap_or(false)
            {
                length += 1;
            }

            let ansi_string = String::from_utf8(
                ansi_strings_buf[(ansi_string_offset + length + 1) as usize..].to_vec(),
            )
            .map_err(|_| Error::invalid_file("Invalid ANSI string".to_string()))?;
            ansi_strings.push(ansi_string);
        }

        let mut wide_strings_buf = vec![0u8; (wide_strings_size * 2) as usize];
        asset.read_exact(&mut wide_strings_buf)?;

        let mut wide_strings = Vec::new();

        for wide_string_offset in wide_string_offsets {
            let mut length = 0;
            while wide_strings_buf
                .get((wide_string_offset + length) as usize)
                .map(|e| *e != 0)
                .unwrap_or(false)
                && wide_strings_buf
                    .get((wide_string_offset + length + 1) as usize)
                    .map(|e| *e != 0)
                    .unwrap_or(false)
            {
                length += 2;
            }

            let wide_string = String::from_utf16(
                &wide_strings_buf[(wide_string_offset + length) as usize..]
                    .chunks(2)
                    .map(|e| u16::from_le_bytes([e[0], e[1]]))
                    .collect::<Vec<_>>(),
            )?;
            wide_strings.push(wide_string);
        }

        let numberless_pairs = asset
            .read_array_with_length(numberless_pairs_count, |asset: &mut Reader| {
                NumberlessPair::new(asset)
            })?;

        let pairs = asset
            .read_array_with_length(pairs_count, |asset: &mut Reader| NumberedPair::new(asset))?;

        asset.seek(SeekFrom::Current(4))?; // END_MAGIC

        Ok(Self {
            pairs,
            numberless_pairs,
            ansi_strings,
            wide_strings,
            numberless_names,
            names,
            numberless_export_paths,
            export_paths,
            texts,
        })
    }
}
