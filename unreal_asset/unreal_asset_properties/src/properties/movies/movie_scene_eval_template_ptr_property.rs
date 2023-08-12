//! Movie scene evaluation template pointer property

use crate::properties::property_prelude::*;

use crate::{
    cast,
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::generate_unversioned_header,
    properties::{str_property::StrProperty, Property, PropertyDataTrait, PropertyTrait},
    reader::archive_reader::ArchiveReader,
    types::fname::FName,
    unversioned::{ancestry::Ancestry, header::UnversionedHeader},
};

/// Movie scene evaluation template pointer property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvalTemplatePtrProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: Vec<Property>,
}
impl_property_data_trait!(MovieSceneEvalTemplatePtrProperty);

impl MovieSceneEvalTemplatePtrProperty {
    /// Read a `MovieSceneEvalTemplatePtrProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let mut value: Vec<Property> = Vec::new();

        let type_name_fname = asset.add_fname("TypeName");
        let new_ancestry = ancestry.with_parent(name.clone());
        let type_name = StrProperty::new(
            asset,
            type_name_fname,
            new_ancestry.clone(),
            include_header,
            0,
        )?;

        if type_name.value.is_some() {
            value.push(type_name.into());
            let mut unversioned_header = UnversionedHeader::new(asset)?;
            while let Some(data) = Property::new(
                asset,
                new_ancestry.clone(),
                unversioned_header.as_mut(),
                true,
            )? {
                value.push(data);
            }
        }

        Ok(MovieSceneEvalTemplatePtrProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEvalTemplatePtrProperty {
    fn write<Writer: crate::reader::archive_writer::ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        let mut had_typename = false;

        let (unversioned_header, sorted_properties) =
            match generate_unversioned_header(asset, &self.value, &self.name)? {
                Some((a, b)) => (Some(a), Some(b)),
                None => (None, None),
            };

        if let Some(unversioned_header) = unversioned_header {
            unversioned_header.write(asset)?;
        }

        let properties = sorted_properties.as_ref().unwrap_or(&self.value);

        for property in properties.iter() {
            if property.get_name() == "TypeName" {
                let str_property: &StrProperty = cast!(Property, StrProperty, property)
                    .ok_or_else(|| {
                        Error::no_data("TypeName property is not StrProperty".to_string())
                    })?;
                had_typename = str_property.value.is_some();
                asset.write_fstring(str_property.value.as_deref())?;
            } else {
                Property::write(property, asset, true)?;
            }
        }

        if had_typename && !asset.has_unversioned_properties() {
            let none_fname = asset.add_fname("None");
            asset.write_fname(&none_fname)?;
        }

        Ok((asset.position() - begin) as usize)
    }
}
