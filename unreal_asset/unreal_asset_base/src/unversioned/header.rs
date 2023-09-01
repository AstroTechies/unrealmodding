//! Unversioned .usmap header

use bitvec::prelude::*;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::types::PackageIndexTrait;
use crate::Error;

/// Unversioned header fragment
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UnversionedHeaderFragment {
    // todo: maybe those are actually i8?
    /// Number of properties to skip before values
    pub skip_num: u8,
    /// Number of subsequent property values stured
    pub value_num: u8,
    /// First element index of this fragment
    pub first_num: u8,
    /// Is this the last header fragment?
    pub is_last: bool,
    /// Has zeros
    pub has_zeros: bool,
}

impl From<u16> for UnversionedHeaderFragment {
    fn from(value: u16) -> Self {
        let skip_num = (value & UnversionedHeaderFragment::SKIP_NUM_MASK) as u8;
        let has_zeros = (value & UnversionedHeaderFragment::HAS_ZEROS_MASK) != 0;
        let value_num = (value >> UnversionedHeaderFragment::VALUE_NUM_SHIFT) as u8;
        let is_last = (value & UnversionedHeaderFragment::IS_LAST_MASK) != 0;

        UnversionedHeaderFragment {
            skip_num,
            value_num,
            first_num: 0,
            is_last,
            has_zeros,
        }
    }
}

impl UnversionedHeaderFragment {
    const SKIP_NUM_MASK: u16 = 0x007fu16;
    const HAS_ZEROS_MASK: u16 = 0x0080u16;
    const VALUE_NUM_SHIFT: u16 = 9;
    const IS_LAST_MASK: u16 = 0x0100u16;

    /// Get last element index of this fragment
    pub fn get_last_num(&self) -> u8 {
        self.first_num + self.value_num - 1
    }

    /// Read an `UnversionedHeaderFragment` from an asset
    pub fn read<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        Ok(UnversionedHeaderFragment::from(asset.read_u16::<LE>()?))
    }

    /// Write an `UnversionedHeaderFragment` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        let has_zero_mask = match self.has_zeros {
            true => UnversionedHeaderFragment::HAS_ZEROS_MASK,
            false => 0,
        };
        let last_mask = match self.is_last {
            true => UnversionedHeaderFragment::IS_LAST_MASK,
            false => 0,
        };

        let packed = self.skip_num as u16
            | has_zero_mask
            | ((self.value_num as u16) << UnversionedHeaderFragment::VALUE_NUM_SHIFT)
            | last_mask;

        asset.write_u16::<LE>(packed)?;

        Ok(())
    }
}

/// List of serialized property indices and which of them are non-zero.
/// Serialized as a stream of 16-bit skip-x keep-y fragments and a zero bitmask.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnversionedHeader {
    /// Fragments
    pub fragments: Vec<UnversionedHeaderFragment>,
    /// Zero mask
    pub zero_mask: BitVec<u8, Lsb0>,
    /// Has non zero values
    pub has_non_zero_values: bool,
    /// Unversioned property index
    pub unversioned_property_index: usize,
    /// Current fragment index
    pub current_fragment_index: usize,
    /// Zero mask index
    pub zero_mask_index: usize,
}

impl UnversionedHeader {
    /// Loads zero mask data from an asset
    fn load_zero_mask_data<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        num_bits: u16,
    ) -> Result<BitVec<u8, Lsb0>, Error> {
        if num_bits <= 8 {
            let mut data = [0u8; 1];
            asset.read_exact(&mut data)?;
            Ok(BitVec::from_vec(data.to_vec()))
        } else if num_bits <= 16 {
            let mut data = [0u8; 2];
            asset.read_exact(&mut data)?;
            Ok(BitVec::from_vec(data.to_vec()))
        } else {
            let num_bytes = ((num_bits + 31) / 32) * 4;
            let mut data = Vec::with_capacity(num_bytes as usize);
            for _ in 0..num_bytes {
                data.push(asset.read_u8()?);
            }
            Ok(BitVec::from_vec(data.to_vec()))
        }
    }

    /// Read `UnversionedHeader` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Option<Self>, Error> {
        if !asset.has_unversioned_properties() {
            return Ok(None);
        }

        let mut fragments = Vec::new();

        let mut first_num = 0;
        let mut zero_mask_num = 0u16;
        let mut unmasked_num = 0u16;

        loop {
            let mut fragment = UnversionedHeaderFragment::read(asset)?;
            fragment.first_num = first_num + fragment.skip_num;
            first_num += fragment.skip_num + fragment.value_num;

            fragments.push(fragment);

            match fragment.has_zeros {
                true => zero_mask_num += fragment.value_num as u16,
                false => unmasked_num += fragment.value_num as u16,
            };

            if fragment.is_last {
                break;
            }
        }

        let (zero_mask, has_non_zero_values) = match zero_mask_num > 0 {
            true => {
                let mask = UnversionedHeader::load_zero_mask_data(asset, zero_mask_num)?;
                let has_non_zero_values = unmasked_num > 0 || mask.iter().all(|e| !*e);
                (mask, has_non_zero_values)
            }
            false => {
                let mask = BitVec::new();
                let has_non_zero_values = unmasked_num > 0;
                (mask, has_non_zero_values)
            }
        };

        let unversioned_property_index = fragments[0].first_num as usize;
        Ok(Some(UnversionedHeader {
            fragments,
            zero_mask,
            has_non_zero_values,
            current_fragment_index: 0,
            unversioned_property_index,
            zero_mask_index: 0,
        }))
    }

    /// Write `UnversionedHeader` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        if !asset.has_unversioned_properties() {
            return Ok(());
        }

        for fragment in &self.fragments {
            fragment.write(asset)?;
        }

        if !self.zero_mask.is_empty() {
            asset.write_all(self.zero_mask.as_raw_slice())?;
        }

        Ok(())
    }
}
