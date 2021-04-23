use crate::{
    data::{GqlEnt, GqlEntFieldAttrMap},
    utils,
};
use entity_macros_data::StructEnt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Path, Type};

pub fn do_derive_ent_filter(
    root: Path,
    ent: StructEnt,
    gql_ent: GqlEnt,
) -> darling::Result<TokenStream> {
    let async_graphql_root = utils::async_graphql_crate()?;
    let entity_gql_root = utils::entity_async_graphql_crate()?;
    let name = &ent.ident;
    let filter_name = format_ident!("Gql{}Filter", name);
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    let ident_id = &ent.id;
    let ident_created = &ent.created;
    let ident_last_updated = &ent.last_updated;

    let gql_map = GqlEntFieldAttrMap::from(gql_ent);

    let (field_struct_field_names, field_struct_fields) = {
        let mut struct_field_names = Vec::new();
        let mut struct_fields = Vec::new();

        for f in &ent.fields {
            struct_field_names.push(&f.name);

            let fname = &f.name;
            let doc_str = format!("Filter by {}'s {} field", name, fname);
            let pred_ty: Type = if gql_map.is_field_untyped(&f.name) {
                parse_quote!(#entity_gql_root::GqlPredicate_Value)
            } else if let Some(ty) = gql_map.get_field_explicit_type_str(&f.name)? {
                parse_quote!(::std::boxed::Box<#ty>)
            } else {
                let ty = format_ident!(
                    "GqlPredicate_{}",
                    utils::type_to_ident(&utils::get_innermost_type(&f.ty))
                        .expect("Failed to convert type to ident")
                );
                parse_quote!(::std::boxed::Box<#entity_gql_root::#ty>)
            };
            struct_fields.push(quote! {
                #[doc = #doc_str]
                #fname: ::std::option::Option<#pred_ty>
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

            // Attempt to determine the filter's type by the following:
            // 1. If specified as untyped, use our standard untyped filter
            // 2. If specified with explicit type, use it
            // 3. Otherwise, fall back to Gql<NAME>Filter as name
            //
            // NOTE: To avoid infinite recursion, any typed filter must be
            //       wrapped in a box
            let filter_ty: Type = if gql_map.is_field_untyped(&e.name) {
                parse_quote!(#entity_gql_root::GqlEntFilter)
            } else if let Some(ty) = gql_map.get_field_explicit_type_str(&e.name)? {
                parse_quote!(::std::boxed::Box<#ty>)
            } else {
                let ty = format_ident!(
                    "Gql{}Filter",
                    utils::type_to_ident(&e.ent_ty).expect("Failed to convert type to ident")
                );
                parse_quote!(::std::boxed::Box<#ty>)
            };

            struct_fields.push(quote! {
                #[doc = #doc_str]
                #ename: ::std::option::Option<#filter_ty>
            });
        }

        (struct_field_names, struct_fields)
    };

    Ok(quote! {
        #[derive(::std::clone::Clone, #async_graphql_root::InputObject)]
        #[graphql(rename_fields = "snake_case")]
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

        impl #impl_generics ::std::convert::From<::std::boxed::Box<#filter_name #ty_generics>>
            for #root::Query #where_clause
        {
            /// Converts from heap by cloning inner filter and converting it
            /// to the generic query
            fn from(x: ::std::boxed::Box<#filter_name #ty_generics>) -> Self {
                Self::from(::std::clone::Clone::clone(::std::convert::AsRef::as_ref(&x)))
            }
        }

        impl #impl_generics ::std::convert::From<#filter_name #ty_generics>
            for #root::Query #where_clause
        {
            fn from(x: #filter_name #ty_generics) -> Self {
                let mut query = <#root::Query as ::std::default::Default>::default();

                if let ::std::option::Option::Some(pred) = x.#ident_id {
                    query.add_filter(#root::Filter::where_id(#root::Predicate::from(pred)));
                }

                if let ::std::option::Option::Some(pred) = x.#ident_created {
                    query.add_filter(#root::Filter::where_created(#root::Predicate::from(pred)));
                }

                if let ::std::option::Option::Some(pred) = x.#ident_last_updated {
                    query.add_filter(#root::Filter::where_last_updated(#root::Predicate::from(pred)));
                }

                #(
                    if let ::std::option::Option::Some(pred) = x.#field_struct_field_names {
                        query.add_filter(#root::Filter::where_field(
                            ::std::stringify!(#field_struct_field_names),
                            pred,
                        ));
                    }
                )*

                #(
                    if let ::std::option::Option::Some(filter) = x.#edge_struct_field_names {
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
