use crate::{
    data::r#struct::{Ent, EntEdge, EntEdgeKind},
    utils,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Path, Type};

pub fn do_derive_ent_typed_edges(root: Path, ent: Ent) -> darling::Result<TokenStream> {
    let name = &ent.ident;
    let mut edge_methods: Vec<TokenStream> = Vec::new();
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    for edge in ent.edges {
        edge_methods.push(fn_typed_id_getter(&edge)?);
        edge_methods.push(fn_typed_id_setter(&edge));
        edge_methods.push(fn_typed_load_edge(&root, &edge));
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#edge_methods)*
        }
    })
}

fn fn_typed_id_getter(edge: &EntEdge) -> darling::Result<TokenStream> {
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
            edge.wrap,
        ),
        EntEdgeKind::One => fn_typed_load_edge_of_one(
            root,
            &format_ident!("load_{}", edge.name),
            &edge.name,
            &edge.ent_ty,
            edge.wrap,
        ),
        EntEdgeKind::Many => fn_typed_load_edge_of_many(
            root,
            &format_ident!("load_{}", edge.name),
            &edge.name,
            &edge.ent_ty,
            edge.wrap,
        ),
    }
}

fn fn_typed_load_edge_of_maybe(
    root: &Path,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
    wrap: bool,
) -> TokenStream {
    let filter_map = if wrap {
        quote!(<#edge_type as #root::EntWrapper>::wrap_ent(ent))
    } else {
        quote!(ent.to_ent::<#edge_type>())
    };

    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<::std::option::Option<#edge_type>> {
            let ents = #root::Ent::load_edge(self, ::std::stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> = ::std::iter::Iterator::collect(
                ::std::iter::Iterator::filter_map(
                    ::std::iter::IntoIterator::into_iter(ents),
                    |ent| #filter_map,
                )
            );
            if typed_ents.len() > 1 {
                ::std::result::Result::Err(#root::DatabaseError::BrokenEdge {
                    name: ::std::string::ToString::to_string(::std::stringify!(#edge_name)),
                })
            } else {
                ::std::result::Result::Ok(
                    ::std::iter::Iterator::next(
                        &mut ::std::iter::IntoIterator::into_iter(typed_ents)
                    )
                )
            }
        }
    }
}

fn fn_typed_load_edge_of_one(
    root: &Path,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
    wrap: bool,
) -> TokenStream {
    let filter_map = if wrap {
        quote!(<#edge_type as #root::EntWrapper>::wrap_ent(ent))
    } else {
        quote!(ent.to_ent::<#edge_type>())
    };

    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<#edge_type> {
            let ents = #root::Ent::load_edge(self, ::std::stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> =
                ::std::iter::Iterator::collect(
                    ::std::iter::Iterator::filter_map(
                        ::std::iter::IntoIterator::into_iter(ents),
                        |ent| #filter_map,
                    )
                );
            if typed_ents.len() != 1 {
                ::std::result::Result::Err(#root::DatabaseError::BrokenEdge {
                    name: ::std::string::ToString::to_string(::std::stringify!(#edge_name)),
                })
            } else {
                ::std::result::Result::Ok(
                    ::std::iter::Iterator::next(
                        &mut ::std::iter::IntoIterator::into_iter(typed_ents)
                    ).unwrap()
                )
            }
        }
    }
}

fn fn_typed_load_edge_of_many(
    root: &Path,
    method_name: &Ident,
    edge_name: &Ident,
    edge_type: &Type,
    wrap: bool,
) -> TokenStream {
    let filter_map = if wrap {
        quote!(<#edge_type as #root::EntWrapper>::wrap_ent(ent))
    } else {
        quote!(ent.to_ent::<#edge_type>())
    };

    quote! {
        pub fn #method_name(&self) -> #root::DatabaseResult<::std::vec::Vec<#edge_type>> {
            let ents = #root::Ent::load_edge(self, ::std::stringify!(#edge_name))?;
            let typed_ents: ::std::vec::Vec<#edge_type> =
                ::std::iter::Iterator::collect(
                    ::std::iter::Iterator::filter_map(
                        ::std::iter::IntoIterator::into_iter(ents),
                        |ent| #filter_map,
                    )
                );
            ::std::result::Result::Ok(typed_ents)
        }
    }
}
