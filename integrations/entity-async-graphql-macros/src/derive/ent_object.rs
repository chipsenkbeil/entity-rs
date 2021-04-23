use crate::utils;
use entity_macros_data::{StructEnt, StructEntEdgeKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

pub fn do_derive_ent_object(root: Path, ent: StructEnt) -> darling::Result<TokenStream> {
    let async_graphql_root = utils::async_graphql_crate()?;
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    let id_fn = {
        let gql_name = ent.id.to_string();
        let method_name = format_ident!("gql_{}", ent.id);
        quote! {
            #[graphql(name = #gql_name)]
            async fn #method_name(&self) -> #root::Id {
                <Self as #root::Ent>::id(self)
            }
        }
    };

    let type_fn = {
        quote! {
            #[graphql(name = "type")]
            async fn gql_type(&self) -> &::std::primitive::str {
                <Self as #root::Ent>::r#type(self)
            }
        }
    };

    let created_fn = {
        let gql_name = ent.created.to_string();
        let method_name = format_ident!("gql_{}", ent.created);
        quote! {
            #[graphql(name = #gql_name)]
            async fn #method_name(&self) -> ::std::primitive::u64 {
                <Self as #root::Ent>::created(self)
            }
        }
    };

    let last_updated_fn = {
        let gql_name = ent.last_updated.to_string();
        let method_name = format_ident!("gql_{}", ent.last_updated);
        quote! {
            #[graphql(name = #gql_name)]
            async fn #method_name(&self) -> ::std::primitive::u64 {
                <Self as #root::Ent>::last_updated(self)
            }
        }
    };

    let field_fns = {
        let mut fns = Vec::new();

        for f in &ent.fields {
            let gql_name = f.name.to_string();
            let method_name = format_ident!("gql_{}", f.name);
            let ret_ty = &f.ty;
            let struct_field_name = &f.name;
            fns.push(quote! {
                #[graphql(name = #gql_name)]
                async fn #method_name(&self) -> &#ret_ty {
                    &self.#struct_field_name
                }
            });
        }

        fns
    };

    let edge_fns = {
        let mut fns = Vec::new();

        for e in &ent.edges {
            let gql_name = match e.kind {
                StructEntEdgeKind::Maybe | StructEntEdgeKind::One => format!("id_for_{}", e.name),
                StructEntEdgeKind::Many => format!("ids_for_{}", e.name),
            };
            let method_name = format_ident!("gql_{}", e.name);
            let ret_ty = &e.ty;
            let struct_field_name = &e.name;
            fns.push(quote! {
                #[graphql(name = #gql_name)]
                async fn #method_name(&self) -> &#ret_ty {
                    &self.#struct_field_name
                }
            });
        }

        fns
    };

    // TODO: Rewrite to not depend on EntTypedEdges being derived for access
    //       to the proper method
    let load_edge_fns = {
        let mut fns = Vec::new();

        for e in &ent.edges {
            let gql_name = e.name.to_string();
            let method_name = format_ident!("gql_load_{}", e.name);
            let ent_ty = &e.ent_ty;
            let ret_ty = match e.kind {
                StructEntEdgeKind::Maybe => quote!(::std::option::Option<#ent_ty>),
                StructEntEdgeKind::One => quote!(#ent_ty),
                StructEntEdgeKind::Many => quote!(::std::vec::Vec<#ent_ty>),
            };
            let load_method_name = format_ident!("load_{}", e.name);
            fns.push(quote! {
                #[graphql(name = #gql_name)]
                async fn #method_name(&self) -> #async_graphql_root::Result<#ret_ty> {
                    self.#load_method_name().map_err(|x|
                        #async_graphql_root::Error::new(::std::string::ToString::to_string(&x))
                    )
                }
            });
        }

        fns
    };

    Ok(quote! {
        #[#async_graphql_root::Object]
        impl #impl_generics #name #ty_generics #where_clause {
            #id_fn
            #type_fn
            #created_fn
            #last_updated_fn
            #(#field_fns)*
            #(#edge_fns)*
            #(#load_edge_fns)*
        }
    })
}
