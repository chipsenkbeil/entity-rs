mod builder;
mod edge;
mod field;
mod ient;
mod info;
mod query;

pub use info::{EntEdge, EntEdgeDeletionPolicy, EntEdgeKind, EntField, EntInfo};

use super::utils;
use heck::ShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::convert::TryFrom;
use syn::DeriveInput;

pub fn do_derive_ent(root: TokenStream, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let const_type_name = format_ident!("{}_TYPE", name.to_string().to_shouty_snake_case());
    let ent_info = EntInfo::try_from(&input)?;

    // Define a constant with a string representing the unique type of the ent
    let const_type_t = quote! {
        #vis const #const_type_name: &str = concat!(module_path!(), "::", stringify!(#name));
    };

    // Unless we have the attribute ent(no_builder), we will add an additional
    // struct of <name>Builder that provides a convenient way to build
    // an ent struct one field at a time
    let builder_t = if utils::has_outer_ent_attr(&input.attrs, "no_builder") {
        quote! {}
    } else {
        builder::impl_ent_builder(&root, &input)?
    };

    // Unless we have the attribute ent(no_query), we will add an additional
    // struct of <name>Query that provides a convenient way to build
    // a typed ent query
    let query_t = if utils::has_outer_ent_attr(&input.attrs, "no_query") {
        quote! {}
    } else {
        query::impl_ent_query(&root, name, vis, generics, &const_type_name, &ent_info)?
    };

    // Unless we have the attribute ent(no_typed_methods), we will add an additional
    // impl that provides loading of specific edges to corresponding types
    let typed_methods_t = if utils::has_outer_ent_attr(&input.attrs, "no_typed_methods") {
        quote! {}
    } else {
        let edge_methods_t =
            edge::impl_typed_edge_methods(&root, &name, generics, &ent_info.edges)?;
        let field_methods_t = field::impl_typed_field_methods(&name, generics, &ent_info.fields);
        quote! {
            #edge_methods_t
            #field_methods_t
        }
    };

    // Implement the IEnt interface with optional typetag if we have the
    // attribute ent(typetag)
    let ent_t = ient::impl_ient(
        &root,
        name,
        generics,
        &ent_info,
        &const_type_name,
        utils::has_outer_ent_attr(&input.attrs, "typetag"),
    )?;

    Ok(quote! {
        #const_type_t
        #ent_t
        #typed_methods_t
        #builder_t
        #query_t
    })
}
