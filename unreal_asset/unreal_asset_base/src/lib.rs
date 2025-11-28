#![deny(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(unexpected_cfgs)]
#![allow(mismatched_lifetime_syntaxes)]
#![allow(clippy::needless_lifetimes)]

//! unreal_asset crate base members

pub mod compression;
pub mod containers;
pub mod crc;
pub mod custom_version;
pub mod engine_version;
pub mod enums;
pub mod error;
pub use error::Error;
pub mod flags;
pub mod import;
pub use import::Import;
pub mod object_version;
pub mod reader;
pub mod types;
pub mod unversioned;

pub use unreal_asset_proc_macro::FNameContainer;
pub use unreal_helpers::Guid;

/// Cast a Property/Export to a more specific type
///
/// # Examples
///
/// ```no_run,ignore
/// use unreal_asset::{
///     cast,
///     properties::{
///         Property,
///         int_property::DoubleProperty,
///     },
/// };
/// let a: Property = ...;
/// let b: &DoubleProperty = cast!(Property, DoubleProperty, &a).unwrap();
/// ```
#[macro_export]
macro_rules! cast {
    ($namespace:ident, $type:ident, $field:expr) => {
        match $field {
            $namespace::$type(e) => Some(e),
            _ => None,
        }
    };
}
