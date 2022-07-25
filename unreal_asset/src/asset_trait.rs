use std::{
    collections::HashMap,
    io::{self, SeekFrom},
};

use crate::{
    custom_version::{CustomVersion, CustomVersionTrait},
    unreal_types::{FName, PackageIndex},
    Import,
};

pub trait AssetTrait {
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>;
    fn position(&self) -> u64;
    fn set_position(&mut self, pos: u64);
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64>;

    fn get_map_key_override<'a>(&'a self) -> &'a HashMap<String, String>;
    fn get_map_value_override<'a>(&'a self) -> &'a HashMap<String, String>;

    fn get_engine_version(&self) -> i32;

    fn get_import<'a>(&'a self, index: PackageIndex) -> Option<&'a Import>;
    fn get_export_class_type<'a>(&'a self, index: PackageIndex) -> Option<FName>;
}
