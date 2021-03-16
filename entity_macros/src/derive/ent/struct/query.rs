use crate::{data::r#struct::Ent, utils};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, Path, Type};

pub fn do_derive_ent_query(root: Path, ent: Ent) -> darling::Result<TokenStream> {
    let mut errors = Vec::new();
    let name = &ent.ident;
    let vis = &ent.vis;
    let query_name = format_ident!("{}Query", name);
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let ty_phantoms: Vec<Type> = ent
        .generics
        .type_params()
        .map(|tp| {
            let tp_ident = &tp.ident;
            parse_quote!(::std::marker::PhantomData<#tp_ident>)
        })
        .collect();
    let default_phantoms: Vec<Expr> = (0..ent.generics.type_params().count())
        .map(|_| parse_quote!(::std::marker::PhantomData))
        .collect();

    let mut methods: Vec<TokenStream> = Vec::new();

    let method_name = format_ident!("where_{}", ent.id);
    methods.push(quote! {
        #[doc = "Filters to return all ents where id passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<#root::Id>) -> Self {
            Self(self.0.where_id(p), #(#default_phantoms),*)
        }
    });

    let method_name = format_ident!("where_{}", ent.created);
    methods.push(quote! {
        #[doc = "Filters to return all ents where created timestamp passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<::std::primitive::u64>) -> Self {
            Self(self.0.where_created(p), #(#default_phantoms),*)
        }
    });

    let method_name = format_ident!("where_{}", ent.last_updated);
    methods.push(quote! {
        #[doc = "Filters to return all ents where last updated timestamp passes the given predicate"]
        pub fn #method_name(self, p: #root::TypedPredicate<::std::primitive::u64>) -> Self {
            Self(self.0.where_last_updated(p), #(#default_phantoms),*)
        }
    });

    for f in &ent.fields {
        let name = &f.name;
        let ty = &f.ty;

        let method_name = format_ident!("where_{}", name);
        let predicate_type = if utils::is_map_type(ty) {
            let value_ty = match ty {
                Type::Path(x) if !x.path.segments.is_empty() => {
                    match utils::get_inner_type_from_segment(x.path.segments.last().unwrap(), 1, 2)
                    {
                        Ok(x) => x,
                        Err(x) => {
                            errors.push(x);
                            continue;
                        }
                    }
                }
                _ => {
                    errors.push(darling::Error::custom("Empty path encountered"));
                    continue;
                }
            };
            quote! { #root::MapTypedPredicate<#value_ty, #ty> }
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
                    self.0.where_field(::std::stringify!(#name), p),
                    #(#default_phantoms),*
                )
            }
        });
    }

    for e in &ent.edges {
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
                <Self as ::std::convert::From<#root::Query>>::from(
                    <Self as ::std::default::Default>::default().0.where_edge(
                        ::std::stringify!(#name),
                        #root::Filter::Id(#root::TypedPredicate::equals(
                            #root::Ent::id(ent)
                        )),
                    )
                )
            }
        });

        let method_name = format_ident!("query_{}", name);

        // NOTE: We attempt to use the specified query type, defaulting to
        //       <NAME>Query if not specified
        let edge_query_ty: Type = if let Some(ty) = e.ent_query_ty.as_ref() {
            parse_quote!(#ty)
        } else {
            let ident = format_ident!(
                "{}Query",
                utils::type_to_ident(ent_ty).expect("Bad edge ent type")
            );
            parse_quote!(#ident)
        };
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
                <#edge_query_ty as ::std::convert::From<#root::Query>>::from(
                    self.0.where_into_edge(::std::stringify!(#name))
                )
            }
        });
    }

    let default_doc_str = format!("Creates new query that selects all {} by default", name);

    let token_stream = quote! {
        #[derive(::std::clone::Clone, ::std::fmt::Debug)]
        #[automatically_derived]
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
                <Self as ::std::convert::From<#root::Query>>::from(
                    #root::Query::default().where_type(
                        #root::TypedPredicate::equals(
                            ::std::string::ToString::to_string(
                                <#name #ty_generics as #root::EntType>::type_str()
                            )
                        )
                    )
                )
            }
        }

        #[automatically_derived]
        impl #impl_generics #query_name #ty_generics #where_clause {
            /// Creates a new instance of the typed query
            pub fn new() -> Self {
                <Self as ::std::default::Default>::default()
            }

            #(#methods)*
        }

        #[automatically_derived]
        impl #impl_generics #root::EntQuery for #query_name #ty_generics #where_clause {
            type Output = ::std::vec::Vec<#name #ty_generics>;

            fn execute<D: #root::Database>(
                self,
                database: &D,
            ) -> #root::DatabaseResult<Self::Output> {
                #root::DatabaseExt::find_all_typed::<#name #ty_generics>(
                    database,
                    self.0,
                )
            }
        }
    };

    if errors.is_empty() {
        Ok(token_stream)
    } else {
        Err(darling::Error::multiple(errors))
    }
}
