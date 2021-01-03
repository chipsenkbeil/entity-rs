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

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let misc_t = quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Retrieves the ent instance with the specified id from
                /// the global database, returning none if ent not found
                pub fn load(id: #root::Id) -> #root::DatabaseResult<::std::option::Option<Self>> {
                    Self::load_from_db(#root::global::db(), id)
                }

                /// Retrieves the ent instance with the specified id from
                /// the global database, converting none into a missing ent error
                pub fn load_strict(id: #root::Id) -> #root::DatabaseResult<Self> {
                    Self::load_from_db_strict(#root::global::db(), id)
                }

                /// Retrieves the ent instance with the specified id from the
                /// provided database, returning none if ent not found
                pub fn load_from_db(
                    db: #root::WeakDatabaseRc,
                    id: #root::Id,
                ) -> #root::DatabaseResult<::std::option::Option<Self>> {
                    let database = #root::WeakDatabaseRc::upgrade(&db)
                        .ok_or(#root::DatabaseError::Disconnected)?;
                    let maybe_ent = #root::Database::get(
                        ::std::convert::AsRef::<#root::Database>::as_ref(
                            ::std::convert::AsRef::<
                                ::std::boxed::Box<dyn #root::Database>
                            >::as_ref(&database),
                        ),
                        id,
                    )?;

                    let maybe_typed_ent = maybe_ent.and_then(|ent| ent.to_ent::<Self>());

                    ::std::result::Result::Ok(maybe_typed_ent)
                }

                /// Retrieves the ent instance with the specified id from the
                /// provided database, converting none into a missing ent error
                pub fn load_from_db_strict(
                    db: #root::WeakDatabaseRc,
                    id: #root::Id,
                ) -> #root::DatabaseResult<Self> {
                    let maybe_ent = Self::load_from_db(db, id)?;

                    match maybe_ent {
                        ::std::option::Option::Some(ent) =>
                            ::std::result::Result::Ok(ent),
                        ::std::option::Option::None =>
                            ::std::result::Result::Err(#root::DatabaseError::MissingEnt { id }),
                    }
                }
            }
        };

        quote! {
            #edge_methods_t
            #field_methods_t
            #misc_t
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
