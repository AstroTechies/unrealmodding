//! Delegate properties

use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Delegate
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Delegate {
    /// Delegate object
    #[container_ignore]
    pub object: PackageIndex,
    /// Delegate name
    pub delegate: FName,
}

impl Delegate {
    /// Create a new `Delegate` instance
    pub fn new(object: PackageIndex, delegate: FName) -> Self {
        Delegate { object, delegate }
    }
}

/// Delegate property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DelegateProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Delegate value
    pub value: Delegate,
}
impl_property_data_trait!(DelegateProperty);

impl DelegateProperty {
    /// Read a `DelegateProperty` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(DelegateProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: Delegate::new(
                PackageIndex::new(asset.read_i32::<LE>()?),
                asset.read_fname()?,
            ),
        })
    }
}

impl PropertyTrait for DelegateProperty {
    fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        asset.write_i32::<LE>(self.value.object.index)?;
        asset.write_fname(&self.value.delegate)?;

        Ok(size_of::<i32>() * 3)
    }
}

// all multicast delegates serialize the same
macro_rules! impl_multicast {
    ($property_name:ident) => {
        /// $property_name
        #[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
        pub struct $property_name {
            /// Name
            pub name: FName,
            /// Property ancestry
            pub ancestry: Ancestry,
            /// Property guid
            pub property_guid: Option<Guid>,
            /// Property duplication index
            pub duplication_index: i32,
            /// Delegates
            pub value: Vec<Delegate>,
        }
        impl_property_data_trait!($property_name);
        impl $property_name {
            /// Read a `$property_name` from an asset
            pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
                asset: &mut Reader,
                name: FName,
                ancestry: Ancestry,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                let length = asset.read_i32::<LE>()?;
                let mut value = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    value.push(Delegate::new(
                        PackageIndex::new(asset.read_i32::<LE>()?),
                        asset.read_fname()?,
                    ));
                }

                Ok($property_name {
                    name,
                    ancestry,
                    property_guid,
                    duplication_index,
                    value,
                })
            }
        }

        impl PropertyTrait for $property_name {
            fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);

                asset.write_i32::<LE>(self.value.len() as i32)?;
                for entry in &self.value {
                    asset.write_i32::<LE>(entry.object.index)?;
                    asset.write_fname(&entry.delegate)?;
                }
                Ok(size_of::<i32>() + size_of::<i32>() * 3 * self.value.len())
            }
        }
    };
}

impl_multicast!(MulticastDelegateProperty);
impl_multicast!(MulticastSparseDelegateProperty);
impl_multicast!(MulticastInlineDelegateProperty);
