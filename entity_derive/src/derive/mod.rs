mod builder;
mod edge;
mod ent;
mod info;
mod utils;

pub use info::{EntEdge, EntEdgeDeletionPolicy, EntEdgeKind, EntField, EntInfo};

use heck::ShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::convert::TryFrom;
use syn::DeriveInput;

pub fn do_derive_ent(root: TokenStream, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    let vis = &input.vis;
    let const_type_name = format_ident!("{}_TYPE", name.to_string().to_shouty_snake_case());
    let ent_info = EntInfo::try_from(&input)?;

    // Define a constant with a string representing the unique type of the ent
    let const_type_t = quote! {
        #vis const #const_type_name: &str = concat!(module_path!(), "::", stringify!(#name));
    };

    // If we have the attribute ent(builder), we will add an additional
    // struct of <name>Builder that provides a convenient way to build
    // an ent struct one field at a time
    let builder_t = if utils::has_outer_ent_attr(&input.attrs, "builder") {
        builder::impl_ent_builder(&root, &input)?
    } else {
        quote! {}
    };

    // If we have the attribute ent(typed_load_edge), we will add an additional
    // impl that provides loading of specific edges to corresponding types
    let typed_methods_t = if utils::has_outer_ent_attr(&input.attrs, "typed_methods") {
        edge::impl_typed_edge_methods(&root, &name, &ent_info.edges)
    } else {
        quote! {}
    };

    // Implement the IEnt interface with optional typetag if we have the
    // attribute ent(typetag)
    let ent_t = ent::impl_ent(
        &root,
        name,
        &ent_info,
        &const_type_name,
        utils::has_outer_ent_attr(&input.attrs, "typetag"),
    );

    Ok(quote! {
        #const_type_t
        #ent_t
        #typed_methods_t
        #builder_t
    })
}
