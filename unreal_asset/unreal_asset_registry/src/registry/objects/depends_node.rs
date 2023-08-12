//! Asset bundle depends node

use bitvec::{order::Lsb0, prelude::BitVec};
use byteorder::LE;
use lazy_static::lazy_static;

use unreal_helpers::BitVecExt;

use crate::custom_version::FAssetRegistryVersionType;
use crate::error::{Error, RegistryError};
use crate::flags::EDependencyProperty;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::fname::FName;

/// Asset identifier
#[derive(Debug, Clone, Default)]
pub struct AssetIdentifier {
    /// Package name
    pub package_name: Option<FName>,
    /// Primary asset type
    pub primary_asset_type: Option<FName>,
    /// Object name
    pub object_name: Option<FName>,
    /// Value name
    pub value_name: Option<FName>,
}

type LoadedDependencyNodes = (Vec<DependsNode>, Vec<DependsNode>, BitVec<u32, Lsb0>);

impl AssetIdentifier {
    /// Read an `AssetIdentifier` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let field_bits = asset.read_u8()?;
        let package_name = match (field_bits & (1 << 0)) != 0 {
            true => Some(asset.read_fname()?),
            false => None,
        };

        let primary_asset_type = match (field_bits & (1 << 1)) != 0 {
            true => Some(asset.read_fname()?),
            false => None,
        };

        let object_name = match (field_bits & (1 << 2)) != 0 {
            true => Some(asset.read_fname()?),
            false => None,
        };

        let value_name = match (field_bits & (1 << 3)) != 0 {
            true => Some(asset.read_fname()?),
            false => None,
        };

        Ok(Self {
            package_name,
            primary_asset_type,
            object_name,
            value_name,
        })
    }

    /// Write an `AssetIdentifier` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        #[allow(clippy::identity_op)]
        let field_bits = (self.package_name.is_some() as u8) << 0u8
            | (self.primary_asset_type.is_some() as u8) << 1u8
            | (self.object_name.is_some() as u8) << 2u8
            | (self.value_name.is_some() as u8) << 3u8;

        writer.write_u8(field_bits)?;
        if let Some(package_name) = &self.package_name {
            writer.write_fname(package_name)?;
        }
        if let Some(primary_asset_type) = &self.primary_asset_type {
            writer.write_fname(primary_asset_type)?;
        }
        if let Some(object_name) = &self.object_name {
            writer.write_fname(object_name)?;
        }
        if let Some(value_name) = &self.value_name {
            writer.write_fname(value_name)?;
        }
        Ok(())
    }
}

/// Depends node
#[derive(Clone, Debug)]
pub struct DependsNode {
    /// Asset identifier
    pub identifier: AssetIdentifier,

    /// Hard dependencies
    pub hard_dependencies: Vec<DependsNode>,
    /// Soft dependencies
    pub soft_dependencies: Vec<DependsNode>,

    /// Name dependencies
    pub name_dependencies: Vec<DependsNode>,

    /// Hard manage dependencies
    pub hard_manage_dependencies: Vec<DependsNode>,
    /// Soft manage dependencies
    pub soft_manage_dependencies: Vec<DependsNode>,

    /// Referencers
    pub referencers: Vec<DependsNode>,

    /// Package flags
    pub package_flags: Option<BitVec<u32, Lsb0>>,
    /// Manage flags
    pub manage_flags: Option<BitVec<u32, Lsb0>>,

    /// Node index
    index: i32,
    /// Asset registry version
    version: FAssetRegistryVersionType,
}

#[allow(unused)]
const PACKAGE_FLAG_WIDTH: i32 = 3;
#[allow(unused)]
const PACKAGE_FLAG_SET_WIDTH: i32 = 1 << PACKAGE_FLAG_WIDTH;
#[allow(unused)]
const MANAGE_FLAG_WIDTH: i32 = 1;
const MANAGE_FLAG_SET_WIDTH: i32 = 1;

lazy_static! {
    static ref HARD_BIT: u8 = DependsNode::package_properties_to_byte(
        EDependencyProperty::HARD | EDependencyProperty::GAME | EDependencyProperty::BUILD
    );
    static ref SOFT_BIT: u8 = DependsNode::package_properties_to_byte(
        EDependencyProperty::GAME | EDependencyProperty::BUILD
    );
}

#[allow(unused)]
const HARD_MANAGE_BITS: u32 = 0x1;
#[allow(unused)]
const SOFT_MANAGE_BITS: u32 = 0x0;

impl DependsNode {
    /// Package `EDependencyProperty` enum into a byte
    #[allow(clippy::identity_op)] // allow for clarity
    fn package_properties_to_byte(properties: EDependencyProperty) -> u8 {
        (0x1 * (properties & EDependencyProperty::HARD).bits() as u8)
            | (0x2 * (properties & EDependencyProperty::GAME).bits() as u8)
            | (0x4 * (properties & EDependencyProperty::BUILD).bits() as u8)
    }

    /// Read `DependsNode` dependencies
    fn read_dependencies<Reader: ArchiveReader>(
        asset: &mut Reader,
        preallocated_depends_node_buffer: &Vec<DependsNode>,
        flag_set_width: i32,
    ) -> Result<LoadedDependencyNodes, Error> {
        let mut sort_indexes = Vec::new();
        let mut pointer_dependencies = Vec::new();

        let in_dependencies = asset.read_array(|asset: &mut Reader| Ok(asset.read_i32::<LE>()?))?;

        let num_flag_bits = flag_set_width * in_dependencies.len() as i32;
        let num_flag_words = (num_flag_bits + 31) / 32;
        let in_flag_bits = match num_flag_words != 0 {
            true => BitVec::from_vec(
                asset.read_array_with_length(num_flag_words, |asset: &mut Reader| {
                    Ok(asset.read_u32::<LE>()?)
                })?,
            ),
            false => BitVec::<u32, Lsb0>::new(),
        };

        for serialize_index in &in_dependencies {
            if *serialize_index < 0
                || preallocated_depends_node_buffer.len() <= *serialize_index as usize
            {
                return Err(RegistryError::InvalidIndex(*serialize_index).into());
            }

            let depends_node = &preallocated_depends_node_buffer[*serialize_index as usize];
            pointer_dependencies.push(depends_node);
        }

        for i in 0..in_dependencies.len() {
            sort_indexes.push(i as i32);
        }

        sort_indexes.sort_by(|a, b| {
            let cmp =
                pointer_dependencies[*a as usize].index - pointer_dependencies[*b as usize].index;

            cmp.cmp(&0)
        });

        let mut hard_dependencies = Vec::new();
        let mut soft_dependencies = Vec::new();

        for index in &sort_indexes {
            let node = pointer_dependencies[*index as usize];
            let package_flags = node.package_flags.as_ref().ok_or_else(|| {
                Error::invalid_file(
                    "No package flags on asset registry with version >= AddedDependencyFlags"
                        .to_string(),
                )
            })?;

            if package_flags
                .get(*HARD_BIT as usize)
                .as_deref()
                .copied()
                .unwrap_or(false)
            {
                hard_dependencies.push(node.clone());
            } else {
                soft_dependencies.push(node.clone());
            }
        }

        let mut out_flag_bits = BitVec::with_capacity(num_flag_bits as usize);
        for write_index in 0..in_dependencies.len() as i32 {
            let read_index = &sort_indexes[write_index as usize];

            out_flag_bits.set_range_from_range(
                write_index * flag_set_width,
                flag_set_width,
                &in_flag_bits,
                read_index * flag_set_width,
            );
        }

        Ok((hard_dependencies, soft_dependencies, out_flag_bits))
    }

    /// Write `DependsNode` dependencies
    fn write_dependencies<Writer: ArchiveWriter>(
        writer: &mut Writer,
        flag_set_width: i32,
        flags: &BitVec<u32, Lsb0>,
        hard_dependencies: &Vec<DependsNode>,
        soft_dependencies: &Vec<DependsNode>,
    ) -> Result<(), Error> {
        let dependencies_length = hard_dependencies.len() as i32 + soft_dependencies.len() as i32;
        let mut out_flag_bits = BitVec::<u32, Lsb0>::new();

        writer.write_i32::<LE>(dependencies_length)?;

        for (i, hard_dependency) in hard_dependencies.iter().enumerate() {
            writer.write_i32::<LE>(hard_dependency.index)?;

            let index = out_flag_bits.len() as i32;
            out_flag_bits.reserve(flag_set_width as usize);
            out_flag_bits.set_range_from_range(
                index,
                flag_set_width,
                flags,
                i as i32 * flag_set_width,
            );
        }

        let inital_soft_index = hard_dependencies.len() as i32;

        for (i, soft_dependency) in soft_dependencies.iter().enumerate() {
            writer.write_i32::<LE>(soft_dependency.index)?;

            let index = out_flag_bits.len() as i32 + inital_soft_index;
            out_flag_bits.reserve(flag_set_width as usize);
            out_flag_bits.set_range_from_range(
                index,
                flag_set_width,
                flags,
                (i as i32 + inital_soft_index) * flag_set_width,
            );
        }

        let bit_vec_size = ((out_flag_bits.len() + 31) / 32) as i32;
        writer.write_i32::<LE>(bit_vec_size)?;

        for byte in out_flag_bits.chunks(8).map(|chunk| {
            let mut byte = 0u8;
            for i in 0..8 {
                if chunk[i] {
                    byte |= 1 << i;
                }
            }
            byte
        }) {
            writer.write_u8(byte)?;
        }

        Ok(())
    }

    /// Read `DependsNode` dependencies without flags
    fn read_dependencies_no_flags<Reader: ArchiveReader>(
        asset: &mut Reader,
        preallocated_depends_node_buffer: &Vec<DependsNode>,
    ) -> Result<Vec<DependsNode>, Error> {
        let mut pointer_dependencies = Vec::new();
        let in_dependencies = asset.read_array(|asset: &mut Reader| Ok(asset.read_i32::<LE>()?))?;

        for serialize_index in &in_dependencies {
            if *serialize_index < 0
                || preallocated_depends_node_buffer.len() <= *serialize_index as usize
            {
                return Err(RegistryError::InvalidIndex(*serialize_index).into());
            }

            let depends_node = &preallocated_depends_node_buffer[*serialize_index as usize];
            pointer_dependencies.push(depends_node);
        }

        let mut sort_indexes = Vec::new();

        for i in 0..in_dependencies.len() as i32 {
            sort_indexes.push(i);
        }

        sort_indexes.sort_by(|a, b| {
            let cmp =
                pointer_dependencies[*a as usize].index - pointer_dependencies[*b as usize].index;

            cmp.cmp(&0)
        });

        let mut out_dependencies = Vec::with_capacity(in_dependencies.len());
        for index in sort_indexes {
            out_dependencies.push(pointer_dependencies[index as usize].clone());
        }

        Ok(out_dependencies)
    }

    /// Write `DependsNode` dependencies without flags
    fn write_dependencies_no_flags<Writer: ArchiveWriter>(
        writer: &mut Writer,
        dependencies: &Vec<DependsNode>,
    ) -> Result<(), Error> {
        writer.write_i32::<LE>(dependencies.len() as i32)?;
        for dependency in dependencies {
            writer.write_i32::<LE>(dependency.index)?;
        }
        Ok(())
    }

    /// Create a new `DependsNode` instance
    pub fn new(index: i32, version: FAssetRegistryVersionType) -> Self {
        Self {
            identifier: AssetIdentifier::default(),
            hard_dependencies: Vec::new(),
            soft_dependencies: Vec::new(),
            name_dependencies: Vec::new(),
            hard_manage_dependencies: Vec::new(),
            soft_manage_dependencies: Vec::new(),
            referencers: Vec::new(),
            package_flags: None,
            manage_flags: None,
            index,
            version,
        }
    }

    /// Load `DependsNode` dependencies
    pub fn load_dependencies<Reader: ArchiveReader>(
        &mut self,
        asset: &mut Reader,
        preallocated_depends_node_buffer: &Vec<DependsNode>,
    ) -> Result<(), Error> {
        let identifier = AssetIdentifier::new(asset)?;

        let (hard_dependencies, soft_dependencies, package_flags) = Self::read_dependencies(
            asset,
            preallocated_depends_node_buffer,
            PACKAGE_FLAG_SET_WIDTH,
        )?;

        let name_dependencies =
            Self::read_dependencies_no_flags(asset, preallocated_depends_node_buffer)?;

        let (hard_manage_dependencies, soft_manage_dependencies, manage_flags) =
            Self::read_dependencies(
                asset,
                preallocated_depends_node_buffer,
                MANAGE_FLAG_SET_WIDTH,
            )?;

        let referencers =
            Self::read_dependencies_no_flags(asset, preallocated_depends_node_buffer)?;

        self.identifier = identifier;

        self.hard_dependencies = hard_dependencies;
        self.soft_dependencies = soft_dependencies;

        self.package_flags = Some(package_flags);
        self.name_dependencies = name_dependencies;

        self.hard_manage_dependencies = hard_manage_dependencies;
        self.soft_manage_dependencies = soft_manage_dependencies;

        self.manage_flags = Some(manage_flags);
        self.referencers = referencers;

        Ok(())
    }

    /// Save `DependsNode` dependencies
    pub fn save_dependencies<Writer: ArchiveWriter>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        self.identifier.write(writer)?;

        let package_flags = self
            .package_flags
            .as_ref()
            .ok_or_else(|| RegistryError::version("Package flags".to_string(), self.version))?;

        Self::write_dependencies(
            writer,
            PACKAGE_FLAG_SET_WIDTH,
            package_flags,
            &self.hard_dependencies,
            &self.soft_dependencies,
        )?;

        Self::write_dependencies_no_flags(writer, &self.name_dependencies)?;

        let manage_flags = self
            .manage_flags
            .as_ref()
            .ok_or_else(|| RegistryError::version("Manage Flags".to_string(), self.version))?;

        Self::write_dependencies(
            writer,
            MANAGE_FLAG_SET_WIDTH,
            manage_flags,
            &self.hard_manage_dependencies,
            &self.soft_manage_dependencies,
        )?;

        Self::write_dependencies_no_flags(writer, &self.referencers)?;

        Ok(())
    }

    /// Read `DependsNode` array
    fn read_node_array<Reader: ArchiveReader>(
        asset: &mut Reader,
        preallocated_depends_node_buffer: &[DependsNode],
        num: i32,
        nodes: &mut Vec<DependsNode>,
    ) -> Result<(), Error> {
        for _ in 0..num {
            let index = asset.read_i32::<LE>()?;
            if index < 0 || nodes.len() <= index as usize {
                return Err(RegistryError::InvalidIndex(index).into());
            }

            let depends_node = &preallocated_depends_node_buffer[index as usize];
            nodes.push(depends_node.clone());
        }

        Ok(())
    }

    /// Load `DependsNode` dependencies before flags
    pub fn load_dependencies_before_flags<Reader: ArchiveReader>(
        &mut self,
        asset: &mut Reader,
        preallocated_depends_node_buffer: &[DependsNode],
    ) -> Result<(), Error> {
        let identifier = AssetIdentifier::new(asset)?;

        let num_hard = asset.read_i32::<LE>()?;
        let num_soft = asset.read_i32::<LE>()?;
        let num_name = asset.read_i32::<LE>()?;
        let num_soft_manage = asset.read_i32::<LE>()?;
        let num_hard_manage = match self.version >= FAssetRegistryVersionType::AddedHardManage {
            true => asset.read_i32::<LE>()?,
            false => 0,
        };
        let num_referencers = asset.read_i32::<LE>()?;

        let mut name_dependencies = Vec::with_capacity(num_name as usize);
        let mut referencers = Vec::with_capacity(num_referencers as usize);

        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_hard,
            &mut self.hard_dependencies,
        )?;
        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_soft,
            &mut self.soft_dependencies,
        )?;
        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_name,
            &mut name_dependencies,
        )?;
        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_soft_manage,
            &mut self.soft_manage_dependencies,
        )?;
        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_hard_manage,
            &mut self.hard_manage_dependencies,
        )?;
        Self::read_node_array(
            asset,
            preallocated_depends_node_buffer,
            num_referencers,
            &mut referencers,
        )?;

        self.identifier = identifier;
        self.name_dependencies = name_dependencies;
        self.referencers = referencers;

        self.package_flags = None;
        self.manage_flags = None;
        Ok(())
    }

    /// Save `DependsNode` dependencies before flags
    pub fn save_dependencies_before_flags<Writer: ArchiveWriter>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        self.identifier.write(writer)?;

        writer.write_i32::<LE>(self.hard_dependencies.len() as i32)?;
        writer.write_i32::<LE>(self.soft_dependencies.len() as i32)?;
        writer.write_i32::<LE>(self.name_dependencies.len() as i32)?;
        writer.write_i32::<LE>(self.soft_manage_dependencies.len() as i32)?;
        if self.version >= FAssetRegistryVersionType::AddedHardManage {
            writer.write_i32::<LE>(self.hard_manage_dependencies.len() as i32)?;
        }
        writer.write_i32::<LE>(self.referencers.len() as i32)?;

        for hard_dependency in &self.hard_dependencies {
            writer.write_i32::<LE>(hard_dependency.index)?;
        }

        for soft_dependency in &self.soft_dependencies {
            writer.write_i32::<LE>(soft_dependency.index)?;
        }

        for name_dependency in &self.name_dependencies {
            writer.write_i32::<LE>(name_dependency.index)?;
        }

        for soft_manage_dependency in &self.soft_manage_dependencies {
            writer.write_i32::<LE>(soft_manage_dependency.index)?;
        }

        if self.version >= FAssetRegistryVersionType::AddedHardManage {
            for hard_manage_dependency in &self.hard_manage_dependencies {
                writer.write_i32::<LE>(hard_manage_dependency.index)?;
            }
        }

        for referencer in &self.referencers {
            writer.write_i32::<LE>(referencer.index)?;
        }

        Ok(())
    }
}
