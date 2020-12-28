use super::{EntEdge, EntEdgeKind};
use crate::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Generics, Ident, Path, Type};

/// Implements individual typed methods for each of the provided edges for
/// the ent with the given name
pub(crate) fn impl_typed_edge_methods(
    root: &Path,
    name: &Ident,
    generics: &Generics,
    edges: &[EntEdge],
) -> Result<TokenStream, syn::Error> {
    let mut edge_methods: Vec<TokenStream> = Vec::new();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    for edge in edges {
        edge_methods.push(fn_typed_id_getter(&edge)?);
        edge_methods.push(fn_typed_id_setter(&edge));
        edge_methods.push(fn_typed_load_edge(root, &edge));
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#edge_methods)*
        }
    })
}

fn fn_typed_id_getter(edge: &EntEdge) -> Result<TokenStream, syn::Error> {
    let name = &edge.name;
    let ty = &edge.ty;

    let method_name = match edge.kind {
        EntEdgeKind::Maybe | EntEdgeKind::One => format_ident!("{}_id", name),
        EntEdgeKind::Many => format_ident!("{}_ids", name),
    };
    let return_type = match edge.kind {
        EntEdgeKind::Maybe | EntEdgeKind::One => quote! { #ty },
        EntEdgeKind::Many => {
            let inner_t = utils::strip_vec(ty)?;
            quote! { &[#inner_t] }
        }
    };
    let inner_return = match edge.kind {
        EntEdgeKind::Maybe | EntEdgeKind::One => quote! { self.#name },
        EntEdgeKind::Many => quote! { &self.#name },
    };

    Ok(quote! {
        pub fn #method_name(&self) -> #return_type {
            #inner_return
        }
    })
}

fn fn_typed_id_setter(edge: &EntEdge) -> TokenStream {
    let name = &edge.name;
    let ty = &edge.ty;

    let method_name = match edge.kind {
        EntEdgeKind::Maybe | EntEdgeKind::One => format_ident!("set_{}_id", name),
        EntEdgeKind::Many => format_ident!("set_{}_ids", name),
    };

    quote! {
        #[doc = "Updates edge ids, returning old value"]
        pub fn #method_name(&mut self, value: #ty) -> #ty {
            ::std::mem::replace(&mut self.#name, value)
        }
    }
}

fn fn_typed_load_edge(root: &Path, edge: &EntEdge) -> TokenStream {
    match edge.kind {
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
    }
}

fn fn_typed_load_edge_of_maybe(
    root: &Path,
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
    root: &Path,
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
    root: &Path,
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
