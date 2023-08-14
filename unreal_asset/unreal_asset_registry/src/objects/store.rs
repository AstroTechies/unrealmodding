//! Asset registry store

use std::io::SeekFrom;

use byteorder::{ReadBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use unreal_asset_base::{reader::ArchiveReader, types::FName, Error};

/// Value type
#[repr(u32)]
#[derive(IntoPrimitive, TryFromPrimitive)]
pub enum EValueType {
    /// Ansi string
    AnsiString,
    /// Wide string
    WideString,
    /// Numberless FName
    NumberlessName,
    /// FName
    Name,
    /// Numberless export path
    NumberlessExportPath,
    /// Export path
    ExportPath,
    /// Localized text
    LocalizedText,
}

/// Value id
pub struct ValueId {
    /// Value type
    pub value_type: EValueType,
    /// Index
    pub index: i32,
}

const TYPE_BITS: u32 = 3;
const INDEX_BITS: u32 = 32 - TYPE_BITS;

impl ValueId {
    /// Read a `ValueId` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let id = asset.read_u32::<LE>()?;
        let value_type = EValueType::try_from((id << INDEX_BITS) >> INDEX_BITS)?;
        let index = id as i32 >> TYPE_BITS;

        Ok(Self { value_type, index })
    }
}

/// Numbered pair
pub struct NumberedPair {
    /// Key
    pub key: FName,
    /// Value
    pub value: ValueId,
}

impl NumberedPair {
    /// Read a `NumberedPair` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let key = asset.read_fname()?;
        let value = ValueId::new(asset)?;

        Ok(Self { key, value })
    }
}

/// Numberless pair
pub struct NumberlessPair {
    /// Key
    pub key: u32,
    /// Value
    pub value: ValueId,
}

impl NumberlessPair {
    /// Read a `NumberlessPair` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let key = asset.read_u32::<LE>()?;
        let value = ValueId::new(asset)?;

        Ok(Self { key, value })
    }
}

/// Numberless export path
pub struct NumberlessExportPath {
    /// Class
    pub class: u32,
    /// Object
    pub object: u32,
    /// Package
    pub package: u32,
}

impl NumberlessExportPath {
    /// Read a `NumberlessExportPath` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let class = asset.read_u32::<LE>()?;
        let object = asset.read_u32::<LE>()?;
        let package = asset.read_u32::<LE>()?;

        Ok(Self {
            class,
            object,
            package,
        })
    }
}

/// Asset registry export path
pub struct AssetRegistryExportPath {
    /// Class
    pub class: FName,
    /// Object
    pub object: FName,
    /// Package
    pub package: FName,
}

impl AssetRegistryExportPath {
    /// Read an `AssetRegistryExportPath` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
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

/// Load order
#[derive(PartialEq, Eq)]
enum ELoadOrder {
    /// Member
    Member,
    /// Text first
    TextFirst,
}

/// Asset registry store
pub struct Store {
    /// Numbered pairs
    pub pairs: Vec<NumberedPair>,
    /// Numberless pairs
    pub numberless_pairs: Vec<NumberlessPair>,
    /// Ansi strings
    pub ansi_strings: Vec<String>,
    /// Wide strings
    pub wide_strings: Vec<String>,
    /// Numberless names
    pub numberless_names: Vec<u32>,
    /// Numbered names
    pub names: Vec<FName>,
    /// Numberless export paths
    pub numberless_export_paths: Vec<NumberlessExportPath>,
    /// Numbered export paths
    pub export_paths: Vec<AssetRegistryExportPath>,
    /// Texts
    pub texts: Vec<Option<String>>,
}

const OLD_BEGIN_MAGIC: u32 = 0x12345678;
const BEGIN_MAGIC: u32 = 0x12345679;

impl Store {
    /// Get asset registry store load order from magic
    fn get_load_order(magic: u32) -> Result<ELoadOrder, Error> {
        match magic {
            OLD_BEGIN_MAGIC => Ok(ELoadOrder::Member),
            BEGIN_MAGIC => Ok(ELoadOrder::TextFirst),
            _ => Err(Error::invalid_file("Invalid asset store magic".to_string())),
        }
    }

    /// Read a `Store` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let magic = asset.read_u32::<LE>()?;
        let order = Store::get_load_order(magic)?;

        let numberless_names_count: i32 = asset.read_i32::<LE>()?;
        let names_count: i32 = asset.read_i32::<LE>()?;
        let numberless_export_paths_count: i32 = asset.read_i32::<LE>()?;
        let export_paths_count: i32 = asset.read_i32::<LE>()?;
        let texts_count: i32 = asset.read_i32::<LE>()?;
        let ansi_string_offsets_count: i32 = asset.read_i32::<LE>()?;
        let wide_string_offsets_count: i32 = asset.read_i32::<LE>()?;
        let ansi_strings_size: i32 = asset.read_i32::<LE>()?;
        let wide_strings_size: i32 = asset.read_i32::<LE>()?;
        let numberless_pairs_count: i32 = asset.read_i32::<LE>()?;
        let pairs_count: i32 = asset.read_i32::<LE>()?;

        let mut texts = Vec::new();
        if order == ELoadOrder::TextFirst {
            asset.seek(std::io::SeekFrom::Current(4))?;
            texts = asset
                .read_array_with_length(texts_count, |asset: &mut Reader| asset.read_fstring())?;
        }

        let numberless_names = asset
            .read_array_with_length(numberless_names_count, |asset: &mut Reader| {
                Ok(asset.read_u32::<LE>()?)
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
                Ok(asset.read_i32::<LE>()?)
            })?;

        let wide_string_offsets = asset
            .read_array_with_length(wide_string_offsets_count, |asset: &mut Reader| {
                Ok(asset.read_i32::<LE>()?)
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
