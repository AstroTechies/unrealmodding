#![deny(missing_docs)]
//! Unreal asset procedural macros

use proc_macro::TokenStream;

mod fname_container;

extern crate proc_macro;

/// FNameContainer derive macro
///
/// This derive macro is used to grab all FName's inside of a struct
/// and generate a function which can iterate over all of them mutably
#[proc_macro_derive(FNameContainer, attributes(container_ignore, container_nobounds))]
pub fn derive_fname_container(input: TokenStream) -> TokenStream {
    fname_container::derive_fname_container(input)
}
