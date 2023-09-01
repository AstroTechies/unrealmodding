//! Archive reader

use std::collections::HashSet;
use std::io::{self, Read};

use byteorder::{ReadBytesExt, LE};

use crate::crc;
use crate::custom_version::CustomVersion;
use crate::enums;
use crate::enums::ECustomVersionSerializationFormat;
use crate::error::{Error, FNameError};
use crate::object_version::ObjectVersion;
use crate::reader::ArchiveTrait;
use crate::types::{FName, PackageIndexTrait, SerializedNameHeader};
use crate::Guid;

/// A trait that allows reading from an archive in an asset-specific way
pub trait ArchiveReader<Index: PackageIndexTrait>: ArchiveTrait<Index> + Read {
    /// Read a `Guid` property
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error> {
        if self.get_object_version() >= ObjectVersion::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            let has_property_guid = self.read_bool()?;
            if has_property_guid {
                return Ok(Some(self.read_guid()?));
            }
        }
        Ok(None)
    }
    /// Read an `FName`
    fn read_fname(&mut self) -> Result<FName, Error> {
        let index = self.read_i32::<LE>()?;
        let number = self.read_i32::<LE>()?;

        let name_map_size = self
            .get_name_map()
            .get_ref()
            .get_name_map_index_list()
            .len();
        if index < 0 || index >= name_map_size as i32 {
            return Err(FNameError::out_of_range(index, name_map_size).into());
        }

        Ok(self.get_name_map().get_ref().create_fname(index, number))
    }

    /// Read custom version container
    fn read_custom_version_container(
        &mut self,
        format: ECustomVersionSerializationFormat,
        old_container: Option<&[CustomVersion]>,
    ) -> Result<Vec<CustomVersion>, Error> {
        match format {
            ECustomVersionSerializationFormat::Unknown => {
                return Err(Error::invalid_file(String::from(
                    "Cannot read a custom version container with an unknown serialization format",
                )))
            }
            ECustomVersionSerializationFormat::Enums => {
                return Err(Error::unimplemented(String::from(
                    "Custom version container with Enums serialization format is unimplemented",
                )))
            }
            _ => {}
        }

        let mut new_container = Vec::new();
        let mut existing_versions = HashSet::new();

        let num_custom_versions = self.read_i32::<LE>()?;
        for _ in 0..num_custom_versions {
            let custom_version_guid = self.read_guid()?;

            let version_number = self.read_i32::<LE>()?;
            new_container.push(CustomVersion::new(custom_version_guid, version_number));
            existing_versions.insert(custom_version_guid);
        }

        // todo: move to iterator joining
        if let Some(custom_version_container) =
            self.get_mappings().as_ref().map(|e| &e.custom_versions)
        {
            for custom_version in custom_version_container {
                if !existing_versions.contains(&custom_version.guid) {
                    new_container.push(custom_version.clone());
                }
            }
        }

        if let Some(old_container) = old_container {
            for custom_version in old_container {
                if !existing_versions.contains(&custom_version.guid) {
                    new_container.push(custom_version.clone());
                }
            }
        }

        Ok(new_container)
    }

    /// Read `FName` name map string
    fn read_name_map_string(
        &mut self,
        serialized_name_header: Option<SerializedNameHeader>,
    ) -> Result<(String, u32), Error> {
        let string = match serialized_name_header {
            Some(e) => self.read_fstring_name_header(e),
            None => self.read_fstring(),
        }?
        .ok_or_else(|| Error::no_data("name_map_string is None".to_string()))?;

        let hash = match self.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED
            && !string.is_empty()
        {
            true => self.read_u32::<LE>()?,
            false => 0,
        };

        Ok((string, hash))
    }

    /// Read `FName` name batch
    fn read_name_batch(&mut self, verify_hashes: bool) -> Result<(Vec<String>, u64), Error> {
        let num_strings = self.read_i32::<LE>()?;
        if num_strings == 0 {
            return Ok((Vec::new(), 0));
        }

        let _strings_length = self.read_u64::<LE>()?;
        let hash_version = self.read_u64::<LE>()?;

        let hashes = match hash_version {
            hash if hash == enums::HASH_VERSION_CITYHASH64 => {
                let mut hashes = Vec::with_capacity(num_strings as usize);
                for _ in 0..num_strings {
                    hashes.push(self.read_u64::<LE>()?); // cityhash64 of crc::to_lower_string
                }
                Ok(hashes)
            }
            _ => Err(Error::unimplemented(format!(
                "Unimplemented name batch algorithm: {}",
                hash_version
            ))),
        }?;

        let mut name_headers = Vec::with_capacity(num_strings as usize);
        for _ in 0..num_strings {
            name_headers.push(SerializedNameHeader::read(self)?);
        }

        let mut name_batch = Vec::with_capacity(num_strings as usize);

        for name_header in name_headers {
            name_batch.push(self.read_name_map_string(Some(name_header)).map(|e| e.0)?);
        }

        if verify_hashes {
            for (i, entry) in name_batch.iter().enumerate() {
                let hash = match hash_version {
                    hash if hash == enums::HASH_VERSION_CITYHASH64 => {
                        Ok(crc::cityhash64_to_lower(entry))
                    }
                    _ => Err(Error::unimplemented(format!(
                        "Unimplemented name batch algorithm: {}",
                        hash_version
                    ))),
                }?;

                if hash != hashes[i] {
                    return Err(Error::name_batch_hash_mismatch(
                        hashes[i],
                        hash,
                        entry.clone(),
                    ));
                }
            }
        }

        Ok((name_batch, hash_version))
    }

    /// Read an array with specified length
    ///
    /// # Examples
    ///
    /// This reads an array of 12 ints
    /// ```no_run,ignore
    /// use unreal_asset::reader::asset_reader::ArchiveReader;
    /// use byteorder::LE;
    ///
    /// let reader: ArchiveReader<impl PackageIndexTrait> = ...;
    /// let ints = reader.read_array_with_length(12, |e| e.read_i32::<LE>()?)?;
    /// ```
    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let mut array = Vec::with_capacity(length as usize);
        for _ in 0..length {
            array.push(getter(self)?);
        }
        Ok(array)
    }

    /// Read an array with the length being read from this archive
    ///
    /// This reads an `i32` to determine the archive length, then calls the getter N times
    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let length = self.read_i32::<LE>()?;
        self.read_array_with_length(length, getter)
    }

    /// Read an FString
    fn read_fstring(&mut self) -> Result<Option<String>, Error>;
    /// Read an FString with a `SerializedNameHeader`
    fn read_fstring_name_header(
        &mut self,
        serialized_name_header: SerializedNameHeader,
    ) -> Result<Option<String>, Error>;
    /// Read a guid.
    fn read_guid(&mut self) -> io::Result<Guid>;
    /// Read `bool`
    fn read_bool(&mut self) -> io::Result<bool>;
}

/// A macro that allows for quick implementation of [`ArchiveReader`] as a passthrough for the underlying archive
#[macro_export]
macro_rules! passthrough_archive_reader {
    ($passthrough:ident) => {
        #[inline(always)]
        fn read_fstring(&mut self) -> Result<Option<String>, Error> {
            self.$passthrough.read_fstring()
        }

        #[inline(always)]
        fn read_fstring_name_header(
            &mut self,
            serialized_name_header: $crate::types::SerializedNameHeader,
        ) -> Result<Option<String>, Error> {
            self.$passthrough
                .read_fstring_name_header(serialized_name_header)
        }

        #[inline(always)]
        fn read_guid(&mut self) -> std::io::Result<unreal_helpers::Guid> {
            self.$passthrough.read_guid()
        }

        #[inline(always)]
        fn read_bool(&mut self) -> std::io::Result<bool> {
            self.$passthrough.read_bool()
        }
    };
}
