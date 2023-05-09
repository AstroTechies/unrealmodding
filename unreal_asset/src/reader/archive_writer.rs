//! Archive writer

use std::collections::HashSet;
use std::io;

use bitvec::{order::Lsb0, vec::BitVec};
use byteorder::{ByteOrder, LE};

use crate::error::{Error, FNameError, PropertyError};
use crate::object_version::ObjectVersion;
use crate::properties::{Property, PropertyDataTrait};
use crate::reader::archive_trait::ArchiveTrait;
use crate::types::{fname::FName, Guid};
use crate::unversioned::header::{UnversionedHeader, UnversionedHeaderFragment};

/// A trait that allows for writing to an archive in an asset-specific way
pub trait ArchiveWriter: ArchiveTrait {
    /// Write a `Guid` property
    fn write_property_guid(&mut self, guid: &Option<Guid>) -> Result<(), Error> {
        if self.get_object_version() >= ObjectVersion::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            self.write_bool(guid.is_some())?;
            if let Some(ref data) = guid {
                self.write_all(data)?;
            }
        }

        Ok(())
    }
    /// Write an `FName`
    fn write_fname(&mut self, fname: &FName) -> Result<(), Error> {
        match fname {
            FName::Backed {
                index,
                number,
                ty: _,
                name_map: _,
            } => {
                self.write_i32::<LE>(*index)?;
                self.write_i32::<LE>(*number)?;
                Ok(())
            }
            FName::Dummy { value, number } => {
                Err(FNameError::dummy_serialize(value, *number).into())
            }
        }
    }

    /// Generate an unversioned header for an unversioned package
    fn generate_unversioned_header(
        &mut self,
        properties: &[Property],
        parent_name: &FName,
    ) -> Result<Option<(UnversionedHeader, Vec<Property>)>, Error> {
        if !self.has_unversioned_properties() {
            return Ok(None);
        }

        let Some(mappings) = self.get_mappings() else {
            return Ok(None);
        };

        let mut first_global_index = u32::MAX;
        let mut last_global_index = u32::MIN;

        let mut properties_to_process = HashSet::new();
        let mut zero_properties: HashSet<u32> = HashSet::new();

        for property in properties {
            let Some((_, global_index)) = mappings.get_property_with_duplication_index(
                &property.get_name(),
                property.get_ancestry(),
                property.get_duplication_index() as u32,
            ) else {
                return Err(PropertyError::no_mapping(&property.get_name().get_content(), property.get_ancestry()).into());
            };

            if matches!(property, Property::EmptyProperty(_)) {
                zero_properties.insert(global_index);
            }

            first_global_index = first_global_index.min(global_index);
            last_global_index = last_global_index.max(global_index);
            properties_to_process.insert(global_index);
        }

        // Sort properties and generate header fragments
        let mut sorted_properties = Vec::new();

        let mut fragments: Vec<UnversionedHeaderFragment> = Vec::new();
        let mut last_num_before_fragment = 0;

        if !properties_to_process.is_empty() {
            loop {
                let mut has_zeros = false;

                // Find next contiguous properties chunk
                let mut start_index = last_num_before_fragment;
                while !properties_to_process.contains(&start_index)
                    && start_index <= last_global_index
                {
                    start_index += 1;
                }

                if start_index > last_global_index {
                    break;
                }

                // Process contiguous properties chunk
                let mut end_index = start_index;
                while properties_to_process.contains(&end_index) {
                    if zero_properties.contains(&end_index) {
                        has_zeros = true;
                    }

                    // todo: clone might not be needed
                    sorted_properties.push(properties[end_index as usize].clone());
                    end_index += 1;
                }

                // Create extra fragments for this chunk
                let mut skip_num = start_index - last_num_before_fragment - 1;
                let mut value_num = (end_index - 1) - start_index + 1;

                while skip_num > i8::MAX as u32 {
                    fragments.push(UnversionedHeaderFragment {
                        skip_num: i8::MAX as u8,
                        value_num: 0,
                        first_num: 0,
                        is_last: false,
                        has_zeros: false,
                    });
                    skip_num -= i8::MAX as u32;
                }
                while value_num > i8::MAX as u32 {
                    fragments.push(UnversionedHeaderFragment {
                        skip_num: 0,
                        value_num: i8::MAX as u8,
                        first_num: 0,
                        is_last: false,
                        has_zeros: false,
                    });
                    value_num -= i8::MAX as u32;
                }

                // Create the main fragment for this chunk
                let fragment = UnversionedHeaderFragment {
                    skip_num: skip_num as u8,
                    value_num: value_num as u8,
                    first_num: start_index as u8,
                    is_last: false,
                    has_zeros,
                };

                fragments.push(fragment);
                last_num_before_fragment = end_index - 1;
            }
        } else {
            fragments.push(UnversionedHeaderFragment {
                skip_num: usize::min(
                    mappings
                        .get_all_properties(&parent_name.get_content())
                        .len(),
                    i8::MAX as usize,
                ) as u8,
                value_num: 0,
                first_num: 0,
                is_last: true,
                has_zeros: false,
            });
        }

        if let Some(fragment) = fragments.last_mut() {
            fragment.is_last = true;
        }

        let mut has_non_zero_values = false;
        let mut zero_mask = BitVec::<u8, Lsb0>::new();

        for fragment in fragments.iter().filter(|e| e.has_zeros) {
            for i in 0..fragment.value_num {
                let is_zero = zero_properties.contains(&((fragment.first_num + i) as u32));
                if !is_zero {
                    has_non_zero_values = true;
                }
                zero_mask.push(is_zero);
            }
        }

        let unversioned_property_index =
            fragments.first().map(|e| e.first_num).unwrap_or_default() as usize;

        let header = UnversionedHeader {
            fragments,
            zero_mask,
            has_non_zero_values,
            unversioned_property_index,
            current_fragment_index: 0,
            zero_mask_index: 0,
        };

        Ok(Some((header, sorted_properties)))
    }

    /// Write `u8`
    fn write_u8(&mut self, value: u8) -> io::Result<()>;
    /// Write `i8`
    fn write_i8(&mut self, value: i8) -> io::Result<()>;
    /// Write `u16`
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> io::Result<()>;
    /// Write `i16`
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> io::Result<()>;
    /// Write `u32`
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> io::Result<()>;
    /// Write `i32`
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> io::Result<()>;
    /// Write `u64`
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> io::Result<()>;
    /// Write `i64`
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> io::Result<()>;
    /// Write `f32`
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> io::Result<()>;
    /// Write `f64`
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> io::Result<()>;
    /// Write an FString
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error>;
    /// Write all of the bytes in the slice
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
    /// Write `bool`
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}

/// A trait that allows for quick implementation of [`ArchiveWriter`] as a pastthrough trait for the underlying archive
pub trait PassthroughArchiveWriter: ArchiveTrait {
    /// Passthrough archive writer type
    type Passthrough: ArchiveWriter;
    /// Get the passthrough archive writer
    fn get_passthrough(&mut self) -> &mut Self::Passthrough;
}

impl<Writer, Passthrough> ArchiveWriter for Passthrough
where
    Writer: ArchiveWriter,
    Passthrough: PassthroughArchiveWriter<Passthrough = Writer>,
{
    #[inline(always)]
    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.get_passthrough().write_u8(value)
    }

    #[inline(always)]
    fn write_i8(&mut self, value: i8) -> io::Result<()> {
        self.get_passthrough().write_i8(value)
    }

    #[inline(always)]
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> io::Result<()> {
        self.get_passthrough().write_u16::<T>(value)
    }

    #[inline(always)]
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> io::Result<()> {
        self.get_passthrough().write_i16::<T>(value)
    }

    #[inline(always)]
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> io::Result<()> {
        self.get_passthrough().write_u32::<T>(value)
    }

    #[inline(always)]
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> io::Result<()> {
        self.get_passthrough().write_i32::<T>(value)
    }

    #[inline(always)]
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> io::Result<()> {
        self.get_passthrough().write_u64::<T>(value)
    }

    #[inline(always)]
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> io::Result<()> {
        self.get_passthrough().write_i64::<T>(value)
    }

    #[inline(always)]
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> io::Result<()> {
        self.get_passthrough().write_f32::<T>(value)
    }

    #[inline(always)]
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> io::Result<()> {
        self.get_passthrough().write_f64::<T>(value)
    }

    #[inline(always)]
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error> {
        self.get_passthrough().write_fstring(value)
    }

    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.get_passthrough().write_all(buf)
    }

    #[inline(always)]
    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.get_passthrough().write_bool(value)
    }
}
