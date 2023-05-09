//! FName Container derive macro
//!
//! This macro is used to grab all FNames inside of a container and traverse them

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, FieldsNamed, FieldsUnnamed};

/// Ignore attribute name
const IGNORE_ATTRIBUTE: &str = "container_ignore";
/// No trait bounds attribute name
/// This is used to prevent trait bounds resolution cycles
const NO_BOUNDS: &str = "container_nobounds";
/// Trait name
const TRAIT_NAME: &str = "crate::types::fname::FNameContainer";
/// FName path
const FNAME_PATH: &str = "crate::types::fname::FName";

/// FName Container derive macro
pub fn derive_fname_container(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data,
        generics,
        ident: name,
        attrs: atrributes,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let should_generate_bounds = !atrributes.iter().any(|e| e.path().is_ident(NO_BOUNDS));

    let body = match &data {
        syn::Data::Struct(e) => {
            generate_body_for_struct(&name, &generics, e, should_generate_bounds, &[TRAIT_NAME])
        }
        syn::Data::Enum(e) => {
            generate_body_for_enum(&name, &generics, e, should_generate_bounds, &[TRAIT_NAME])
        }
        syn::Data::Union(_) => panic!("This macro cannot be used on unit structs!"),
    };

    TokenStream::from(body)
}

fn generate_body_for_struct(
    name: &syn::Ident,
    generics: &syn::Generics,
    data_struct: &DataStruct,
    should_generate_bounds: bool,
    traits: &[&str],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, old_where_clause) = generics.split_for_impl();

    let new_where_clause = match should_generate_bounds {
        true => {
            let new_where_clause = add_trait_bounds_for_all_struct_fields(data_struct, traits);
            let mut new_where_clause: syn::WhereClause = syn::parse2(new_where_clause).unwrap();
            if let Some(old_where_clause) = old_where_clause {
                new_where_clause
                    .predicates
                    .extend(old_where_clause.predicates.clone());
            }
            quote! { #new_where_clause }
        }
        false => quote! { #old_where_clause },
    };

    let body = match &data_struct.fields {
        syn::Fields::Named(e) => body_for_struct_named_fields(e),
        syn::Fields::Unnamed(e) => body_for_struct_unnamed_fields(e),
        syn::Fields::Unit => quote! {},
    };

    let trait_name: syn::Path = syn::parse_str(TRAIT_NAME).unwrap();
    let fname_path: syn::Path = syn::parse_str(FNAME_PATH).unwrap();

    quote! {
        impl #impl_generics #trait_name for #name #ty_generics #new_where_clause {
            fn traverse_fnames<F: FnMut(&mut #fname_path)>(&mut self, traverse: &mut F) {
                #body
            }
        }
    }
}

fn generate_body_for_enum(
    name: &syn::Ident,
    generics: &syn::Generics,
    data_enum: &DataEnum,
    should_generate_bounds: bool,
    traits: &[&str],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, old_where_clause) = generics.split_for_impl();

    let new_where_clause = match should_generate_bounds {
        true => {
            let new_where_clause = add_trait_bounds_for_all_enum_variants(data_enum, traits);
            let mut new_where_clause: syn::WhereClause = syn::parse2(new_where_clause).unwrap();
            if let Some(old_where_clause) = old_where_clause {
                new_where_clause
                    .predicates
                    .extend(old_where_clause.predicates.clone());
            }
            quote! { #new_where_clause }
        }
        false => quote! { #old_where_clause },
    };

    let body = body_for_enum_variants(data_enum);

    let trait_name: syn::Path = syn::parse_str(TRAIT_NAME).unwrap();
    let fname_path: syn::Path = syn::parse_str(FNAME_PATH).unwrap();

    quote! {
        impl #impl_generics #trait_name for #name #ty_generics #new_where_clause {
            fn traverse_fnames<F: FnMut(&mut #fname_path)>(&mut self, traverse: &mut F) {
                match self {
                    #body
                    _ => {}
                }
            }
        }
    }
}

fn add_trait_bounds_for_all_struct_fields(
    data_struct: &DataStruct,
    traits: &[&str],
) -> proc_macro2::TokenStream {
    let traits: syn::Type = syn::parse_str(&traits.join(" + ")).unwrap();

    let bounds = for_each_field_type(&data_struct.fields, |field_type| {
        quote! {
            #field_type: #traits,
        }
    });

    quote! {
        where #bounds
    }
}

fn add_trait_bounds_for_all_enum_variants(
    data_enum: &DataEnum,
    traits: &[&str],
) -> proc_macro2::TokenStream {
    let traits: syn::Type = syn::parse_str(&traits.join(" + ")).unwrap();

    let bounds =
        proc_macro2::TokenStream::from_iter(data_enum.variants.iter().map(|e| match &e.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                for_each_unnamed_field_type(unnamed_fields, |field_type| {
                    quote! {
                        #field_type: #traits,
                    }
                })
            }
            syn::Fields::Named(_) => {
                panic!("This macro cannot be used on enums with named fields!")
            }
            syn::Fields::Unit => quote! {},
        }));

    quote! {
        where #bounds
    }
}

fn for_each_field_type<F: Fn(&syn::Type) -> proc_macro2::TokenStream>(
    fields: &syn::Fields,
    executor: F,
) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields_named) => for_each_named_field_type(fields_named, executor),
        syn::Fields::Unnamed(fields_unnamed) => {
            for_each_unnamed_field_type(fields_unnamed, executor)
        }
        syn::Fields::Unit => quote! {},
    }
}

fn for_each_named_field_type<F: Fn(&syn::Type) -> proc_macro2::TokenStream>(
    fields_named: &FieldsNamed,
    executor: F,
) -> proc_macro2::TokenStream {
    let streams = fields_named
        .named
        .iter()
        .filter(|e| !e.attrs.iter().any(|e| e.path().is_ident(IGNORE_ATTRIBUTE)))
        .map(|e| executor(&e.ty));

    quote! {
        #(#streams)*
    }
}

fn for_each_unnamed_field_type<F: Fn(&syn::Type) -> proc_macro2::TokenStream>(
    fields_unnamed: &FieldsUnnamed,
    executor: F,
) -> proc_macro2::TokenStream {
    let streams = fields_unnamed
        .unnamed
        .iter()
        .filter(|e| !e.attrs.iter().any(|e| e.path().is_ident(IGNORE_ATTRIBUTE)))
        .map(|e| executor(&e.ty));

    quote! {
        #(#streams)*
    }
}

fn body_for_struct_named_fields(fields_named: &FieldsNamed) -> proc_macro2::TokenStream {
    let streams = fields_named
        .named
        .iter()
        .filter(|e| !e.attrs.iter().any(|e| e.path().is_ident(IGNORE_ATTRIBUTE)))
        .map(|e| {
            let name = e.ident.as_ref().unwrap();
            quote! {
                self.#name.traverse_fnames(traverse)
            }
        });
    quote! {
        #(#streams;)*
    }
}

fn body_for_struct_unnamed_fields(fields_unnamed: &FieldsUnnamed) -> proc_macro2::TokenStream {
    let streams = fields_unnamed
        .unnamed
        .iter()
        .filter(|e| !e.attrs.iter().any(|e| e.path().is_ident(IGNORE_ATTRIBUTE)))
        .enumerate()
        .map(|(index, _)| {
            let index = syn::Index::from(index);
            quote! {
                self.#index.traverse_fnames(traverse)
            }
        });

    quote! {
        #(#streams;)*
    }
}

fn body_for_enum_variants(data_enum: &DataEnum) -> proc_macro2::TokenStream {
    const VARIABLE_NAMES: [&str; 26] = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    let variant_streams = data_enum.variants.iter().map(|e| {
        let name = &e.ident;

        let fields = e
            .fields
            .iter()
            .filter(|e| !e.attrs.iter().any(|e| e.path().is_ident(IGNORE_ATTRIBUTE)))
            .enumerate()
            .map(|(index, _)| format_ident!("{}", VARIABLE_NAMES[index]));

        let fields_ = fields.clone();

        quote! {
            Self::#name #((#fields,))* => {
                #(#fields_.traverse_fnames(traverse);)*
            }
        }
    });

    quote! {
        #(#variant_streams,)*
    }
}
