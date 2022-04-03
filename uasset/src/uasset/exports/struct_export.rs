use crate::uasset::fproperty::FProperty;
use crate::uasset::unreal_types::PackageIndex;

pub struct StructExport {
    super_struct: PackageIndex,
    children: Vec<PackageIndex>,
    loaded_properties: Vec<FProperty>,

}