//! Enum property

use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Enum property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct EnumProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Enum type
    pub enum_type: Option<FName>,
    /// Inner type, used only with unversioned properties
    pub inner_type: Option<FName>,
    /// Enum value
    pub value: Option<FName>,
}
impl_property_data_trait!(EnumProperty);

impl EnumProperty {
    /// Read an `EnumProperty` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let mut enum_type: Option<FName> = None;
        let mut inner_type: Option<FName> = None;
        if asset.has_unversioned_properties() {
            if let Some(enum_data) = asset
                .get_mappings()
                .and_then(|e| e.get_property(&name, &ancestry))
                .and_then(|e| cast!(UsmapPropertyData, UsmapEnumPropertyData, &e.property_data))
            {
                let enum_ty = FName::new_dummy(enum_data.name.clone(), 0);
                let inner_ty =
                    FName::new_dummy(enum_data.inner_property.get_property_type().to_string(), 0);

                if inner_ty == "ByteProperty" {
                    let enum_index = asset.read_u8()?;
                    let info = enum_ty
                        .get_content(|ty| asset.get_mappings().unwrap().enum_map.get_by_key(ty))
                        .ok_or_else(|| {
                            Error::invalid_file(enum_ty.get_content(|ty| {
                                "Missing unversioned info for: ".to_string() + ty
                            }))
                        })?;
                    let value = match enum_index == u8::MAX {
                        true => None,
                        false => Some(FName::new_dummy(info[enum_index as usize].clone(), 0)),
                    };

                    return Ok(EnumProperty {
                        name,
                        ancestry,
                        property_guid: None,
                        duplication_index,
                        enum_type: Some(enum_ty),
                        inner_type: Some(inner_ty),
                        value,
                    });
                }

                enum_type = Some(enum_ty);
                inner_type = Some(inner_ty);
            }
        }

        let property_guid = match include_header {
            true => {
                enum_type = Some(asset.read_fname()?);
                asset.read_property_guid()?
            }
            false => None,
        };
        let value = asset.read_fname()?;

        Ok(EnumProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            enum_type,
            inner_type,
            value: Some(value),
        })
    }
}

impl PropertyTrait for EnumProperty {
    fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        if asset.has_unversioned_properties()
            && self
                .inner_type
                .as_ref()
                .map(|e| e == "ByteProperty")
                .unwrap_or(false)
        {
            self.enum_type
                .as_ref()
                .ok_or_else(|| {
                    Error::no_data("enum_type is None on an unversioned property".to_string())
                })?
                .get_content(|enum_type| {
                    let info = asset
                        .get_mappings()
                        .ok_or_else(PropertyError::no_mappings)?
                        .enum_map
                        .get_by_key(enum_type)
                        .ok_or_else(|| {
                            Error::invalid_file(
                                "Missing unversioned info for: ".to_string() + enum_type,
                            )
                        })?;

                    let enum_index = match self.value.as_ref() {
                        Some(value) => info
                            .iter()
                            .enumerate()
                            .find(|(_, e)| value == *e)
                            .map(|(index, _)| index as u8)
                            .ok_or_else(|| {
                                Error::invalid_file(
                                    "Missing unversioned info for: ".to_string() + enum_type,
                                )
                            })?,
                        None => u8::MAX,
                    };

                    asset.write_u8(enum_index)?;
                    Ok::<(), Error>(())
                })?;
            return Ok(size_of::<u8>());
        }

        if include_header {
            asset.write_fname(
                self.enum_type
                    .as_ref()
                    .ok_or_else(PropertyError::headerless)?,
            )?;
            asset.write_property_guid(self.property_guid.as_ref())?;
        }
        asset.write_fname(self.value.as_ref().unwrap())?;

        Ok(size_of::<i32>() * 2)
    }
}
