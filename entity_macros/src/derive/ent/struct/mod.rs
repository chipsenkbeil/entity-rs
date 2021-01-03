mod builder;
mod data;
mod edge;
mod ent;
mod field;
mod query;

pub use data::{Ent, EntEdge, EntEdgeDeletionPolicy, EntEdgeKind, EntField};

use crate::utils;
use heck::ShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::convert::TryFrom;
use syn::{DeriveInput, Path};

pub fn do_derive_ent(root: Path, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let const_type_name = format_ident!("{}_TYPE", name.to_string().to_shouty_snake_case());
    let ent = Ent::try_from(&input)?;

    // Define a constant with a string representing the unique type of the ent
    let const_type_t = quote! {
        #vis const #const_type_name: &::std::primitive::str = ::std::concat!(
            ::std::module_path!(), "::", ::std::stringify!(#name),
        );
    };

    // Unless we have the attribute ent(no_builder), we will add an additional
    // struct of <name>Builder that provides a convenient way to build
    // an ent struct one field at a time
    let builder_t = if ent.attr.no_builder {
        quote! {}
    } else {
        builder::impl_ent_builder(&root, &input, &ent)?
    };

    // Unless we have the attribute ent(no_query), we will add an additional
    // struct of <name>Query that provides a convenient way to build
    // a typed ent query
    let query_t = if ent.attr.no_query {
        quote! {}
    } else {
        query::impl_ent_query(&root, name, vis, generics, &const_type_name, &ent)?
    };

    // Unless we have the attribute ent(no_typed_methods), we will add an additional
    // impl that provides loading of specific edges to corresponding types
    let typed_methods_t = if ent.attr.no_typed_methods {
        quote! {}
    } else {
        let edge_methods_t = edge::impl_typed_edge_methods(&root, &name, generics, &ent.edges)?;
        let field_methods_t = field::impl_typed_field_methods(&name, generics, &ent.fields);
        quote! {
            #edge_methods_t
            #field_methods_t
        }
    };

    // Implement the Ent interface with optional typetag if we have the
    // attribute ent(typetag)
    let ent_t = ent::impl_ent(&root, name, generics, &ent, &const_type_name)?;

    Ok(quote! {
        #const_type_t
        #ent_t
        #typed_methods_t
        #builder_t
        #query_t
    })
}
