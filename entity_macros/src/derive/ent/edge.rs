use super::{EntEdge, EntEdgeKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

/// Implements individual typed methods for each of the provided edges for
/// the ent with the given name
pub(crate) fn impl_typed_edge_methods(
    root: &TokenStream,
    name: &Ident,
    edges: &[EntEdge],
) -> TokenStream {
    let mut edge_methods: Vec<TokenStream> = Vec::new();

    for edge in edges {
        let load_method = match edge.kind {
            EntEdgeKind::Maybe => fn_typed_load_edge_of_maybe(
                root,
                &format_ident!("load_{}", edge.name),
                &edge.name,
                &edge.ent_ty,
            ),
            EntEdgeKind::One => fn_typed_load_edge_of_one(
                root,
                &format_ident!("load_{}", edge.name),
                &edge.name,
                &edge.ent_ty,
            ),
            EntEdgeKind::Many => fn_typed_load_edge_of_many(
                root,
                &format_ident!("load_{}", edge.name),
                &edge.name,
                &edge.ent_ty,
            ),
        };
        edge_methods.push(load_method);
    }

    quote! {
        #[automatically_derived]
        impl #name {
            #(#edge_methods)*
        }
    }
}

fn fn_typed_load_edge_of_maybe(
    root: &TokenStream,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
) -> TokenStream {
    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<::std::option::Option<#edge_type>> {
            use #root::IEnt;
            let ents = self.load_edge(stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> =
                ents.into_iter().filter_map(|ent|
                    ent.as_any().downcast_ref::<#edge_type>()
                        .map(::std::clone::Clone::clone)
                ).collect();
            if typed_ents.len() > 1 {
                ::std::result::Result::Err(#root::DatabaseError::BrokenEdge {
                    name: stringify!(#edge_name).to_string(),
                })
            } else {
                ::std::result::Result::Ok(typed_ents.into_iter().next())
            }
        }
    }
}

fn fn_typed_load_edge_of_one(
    root: &TokenStream,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
) -> TokenStream {
    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<#edge_type> {
            use #root::IEnt;
            let ents = self.load_edge(stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> =
                ents.into_iter().filter_map(|ent|
                    ent.as_any().downcast_ref::<#edge_type>()
                        .map(::std::clone::Clone::clone)
                ).collect();
            if typed_ents.len() != 1 {
                ::std::result::Result::Err(#root::DatabaseError::BrokenEdge {
                    name: stringify!(#edge_name).to_string(),
                })
            } else {
                ::std::result::Result::Ok(typed_ents.into_iter().next().unwrap())
            }
        }
    }
}

fn fn_typed_load_edge_of_many(
    root: &TokenStream,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
) -> TokenStream {
    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<::std::vec::Vec<#edge_type>> {
            use #root::IEnt;
            let ents = self.load_edge(stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> =
                ents.into_iter().filter_map(|ent|
                    ent.as_any().downcast_ref::<#edge_type>()
                        .map(::std::clone::Clone::clone)
                ).collect();
            ::std::result::Result::Ok(typed_ents)
        }
    }
}
