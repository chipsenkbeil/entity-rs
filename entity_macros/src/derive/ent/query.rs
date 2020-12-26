use super::EntInfo;
use crate::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, Generics, Ident, Type, Visibility};

pub fn impl_ent_query(
    root: &TokenStream,
    name: &Ident,
    vis: &Visibility,
    generics: &Generics,
    const_type_name: &Ident,
    ent_info: &EntInfo,
) -> Result<TokenStream, syn::Error> {
    let query_name = format_ident!("{}Query", name);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let ty_phantoms: Vec<Type> = generics
        .type_params()
        .map(|tp| {
            let tp_ident = &tp.ident;
            parse_quote!(::std::marker::PhantomData<#tp_ident>)
        })
        .collect();
    let default_phantoms: Vec<Expr> = (0..generics.type_params().count())
        .map(|_| parse_quote!(::std::marker::PhantomData))
        .collect();

    let mut methods: Vec<TokenStream> = Vec::new();

    let method_name = format_ident!("where_{}", ent_info.id);
    methods.push(quote! {
        #[doc = "Filters to return all ents where id passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<#root::Id>) -> Self {
            Self(self.0.where_id(p), #(#default_phantoms),*)
        }
    });

    let method_name = format_ident!("where_{}", ent_info.created);
    methods.push(quote! {
        #[doc = "Filters to return all ents where created timestamp passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<u64>) -> Self {
            Self(self.0.where_created(p), #(#default_phantoms),*)
        }
    });

    let method_name = format_ident!("where_{}", ent_info.last_updated);
    methods.push(quote! {
        #[doc = "Filters to return all ents where last updated timestamp passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<u64>) -> Self {
            Self(self.0.where_last_updated(p), #(#default_phantoms),*)
        }
    });

    for f in &ent_info.fields {
        let name = &f.name;
        let ty = &f.ty;

        let method_name = format_ident!("where_{}", name);
        let predicate_type = if utils::is_map_type(ty) {
            quote! { #root::MapTypedPredicate<#ty> }
        } else {
            quote! { #root::TypedPredicate<#ty> }
        };

        let doc_string = format!(
            "Filters to return all ents where the field \"{}\" passes the given predicate",
            name
        );

        methods.push(quote! {
            #[doc = #doc_string]
            pub fn #method_name(self, p: #predicate_type) -> Self {
                Self(
                    self.0.where_field(stringify!(#name), p),
                    #(#default_phantoms),*
                )
            }
        });
    }

    for e in &ent_info.edges {
        let name = &e.name;
        let ent_ty = &e.ent_ty;

        let method_name = format_ident!("query_from_{}", name);
        let doc_string = format!(
            "Filters to return all ents with edge \"{}\" referencing this ent",
            name,
        );

        methods.push(quote! {
            #[doc = #doc_string]
            pub fn #method_name(ent: &#ent_ty) -> Self {
                use #root::IEnt;
                use ::std::default::Default;
                use ::std::convert::From;
                Self::from(Self::default().0.where_edge(
                    stringify!(#name),
                    #root::Filter::Id(#root::TypedPredicate::equals(ent.id())),
                ))
            }
        });

        let method_name = format_ident!("query_{}", name);
        let edge_query_ty = format_ident!(
            "{}Query",
            utils::type_to_ident(ent_ty).expect("Bad edge ent type")
        );
        let doc_string = format!(
            concat!(
                "Returns a query for \"{}\" that is pre-filtered to ents ",
                "referenced by ents contained in the current query",
            ),
            name,
        );

        methods.push(quote! {
            #[doc = #doc_string]
            pub fn #method_name(self) -> #edge_query_ty {
                use #root::IEnt;
                use ::std::convert::From;
                #edge_query_ty::from(self.0.where_into_edge(stringify!(#name)))
            }
        });
    }

    let default_doc_str = format!("Creates new query that selects all {} by default", name);

    Ok(quote! {
        #[derive(::std::clone::Clone, ::std::fmt::Debug)]
        #vis struct #query_name #impl_generics(
            #root::Query,
            #(#ty_phantoms),*
        ) #where_clause;

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#query_name #ty_generics> for #root::Query #where_clause {
            /// Converts into an untyped query
            fn from(q: #query_name #ty_generics) -> Self {
                q.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#root::Query> for #query_name #ty_generics #where_clause {
            /// Converts from a raw, untyped query. No checks are made, so if
            /// ents of other types would be returned, they are instead
            /// discarded from the query results.
            fn from(q: #root::Query) -> Self {
                Self(q, #(#default_phantoms),*)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #query_name #ty_generics #where_clause {
            #[doc = #default_doc_str]
            fn default() -> Self {
                use std::convert::From;
                Self::from(
                    #root::Query::default().where_type(
                        #root::TypedPredicate::equals(
                            ::std::string::String::from(#const_type_name)
                        )
                    )
                )
            }
        }

        #[automatically_derived]
        impl #impl_generics #query_name #ty_generics #where_clause {
            #(#methods)*

            #[doc = "Executes query against the given database"]
            pub fn execute<__entity_D: #root::Database>(
                self,
                database: &__entity_D,
            ) -> #root::DatabaseResult<Vec<#name #ty_generics>> {
                use #root::DatabaseExt;
                database.find_all_typed::<#name #ty_generics>(self.0)
            }
        }
    })
}
