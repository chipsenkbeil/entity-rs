use crate::{
    data::r#struct::{Ent, EntEdgeKind},
    utils,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, Ident, Path};

pub fn do_derive_async_graphql_ent(root: Path, ent: Ent) -> darling::Result<TokenStream> {
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
                EntEdgeKind::Maybe | EntEdgeKind::One => format!("id_for_{}", e.name),
                EntEdgeKind::Many => format!("ids_for_{}", e.name),
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
                EntEdgeKind::Maybe => quote!(::std::option::Option<#ent_ty>),
                EntEdgeKind::One => quote!(#ent_ty),
                EntEdgeKind::Many => quote!(::std::vec::Vec<#ent_ty>),
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

pub fn do_derive_async_graphql_ent_filter(root: Path, ent: Ent) -> darling::Result<TokenStream> {
    let async_graphql_root = utils::async_graphql_crate()?;
    let entity_gql_root = quote!(#root::ext::async_graphql);
    let name = &ent.ident;
    let filter_name = format_ident!("Gql{}Filter", name);
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    let ident_id = &ent.id;
    let ident_created = &ent.created;
    let ident_last_updated = &ent.last_updated;

    let (field_struct_field_names, field_struct_fields) = {
        let mut struct_field_names = Vec::new();
        let mut struct_fields = Vec::new();

        for f in &ent.fields {
            struct_field_names.push(&f.name);

            let fname = &f.name;
            let doc_str = format!("Filter by {}'s {} field", name, fname);
            let pred_ident: Ident = if f.ext.async_graphql_filter_untyped {
                Ident::new("GqlPredicate_Value", Span::mixed_site())
            } else {
                format_ident!(
                    "GqlPredicate_{}",
                    utils::type_to_ident(&utils::get_innermost_type(&f.ty))
                        .expect("Failed to convert type to ident")
                )
            };
            struct_fields.push(quote! {
                #[doc = #doc_str]
                #fname: ::std::option::Option<#entity_gql_root::#pred_ident>
            });
        }

        (struct_field_names, struct_fields)
    };

    let (edge_struct_field_names, edge_struct_fields) = {
        let mut struct_field_names = Vec::new();
        let mut struct_fields = Vec::new();

        for e in &ent.edges {
            struct_field_names.push(&e.name);

            let ename = &e.name;
            let doc_str = format!("Filter by {}'s {} edge", name, ename);

            // TODO: Support typed filter by enabling an extension to
            //       specify the type. We have to do this because we don't
            //       have a guarantee that the filter is imported alongside
            //       the ent, so it may not be in the current scope
            let filter_ident: Ident = if e.ext.async_graphql_filter_untyped {
                parse_quote!(#entity_gql_root::GqlEntFilter)
            } else {
                format_ident!(
                    "Gql{}Filter",
                    utils::type_to_ident(&e.ent_ty).expect("Failed to convert type to ident")
                )
            };

            struct_fields.push(quote! {
                #[doc = #doc_str]
                #ename: ::std::option::Option<#filter_ident>
            });
        }

        (struct_field_names, struct_fields)
    };

    Ok(quote! {
        #[derive(::std::clone::Clone, #async_graphql_root::InputObject)]
        pub struct #filter_name #ty_generics #where_clause {
            /// Filter by ent's id
            #ident_id: ::std::option::Option<#entity_gql_root::GqlPredicate_Id>,

            /// Filter by ent's creation timestamp
            #ident_created: ::std::option::Option<#entity_gql_root::GqlPredicate_u64>,

            /// Filter by ent's last updated timestamp
            #ident_last_updated: ::std::option::Option<#entity_gql_root::GqlPredicate_u64>,

            #(#field_struct_fields,)*
            #(#edge_struct_fields,)*
        }

        impl #impl_generics ::std::convert::From<#filter_name #ty_generics>
            for #root::Query #where_clause
        {
            fn from(x: #filter_name #ty_generics) -> Self {
                let mut query = #root::Query::default();

                if let Some(pred) = x.#ident_id {
                    query.add_filter(#root::Filter::where_id(#root::Predicate::from(pred)));
                }

                if let Some(pred) = x.#ident_created {
                    query.add_filter(#root::Filter::where_created(#root::Predicate::from(pred)));
                }

                if let Some(pred) = x.#ident_last_updated {
                    query.add_filter(#root::Filter::where_last_updated(#root::Predicate::from(pred)));
                }

                #(
                    if let Some(pred) = x.#field_struct_field_names {
                        query.add_filter(#root::Filter::where_field(
                            ::std::stringify!(#field_struct_field_names),
                            pred,
                        ));
                    }
                )*

                #(
                    if let Some(filter) = x.#edge_struct_field_names {
                        let edge_query = #root::Query::from(filter);
                        for edge_filter in edge_query {
                            query.add_filter(#root::Filter::where_edge(
                                ::std::stringify!(#edge_struct_field_names),
                                edge_filter,
                            ));
                        }
                    }
                )*

                query
            }
        }
    })
}
