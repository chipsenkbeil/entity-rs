use entity_macros_data::EnumEnt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, Path, Type};

pub fn do_derive_ent_query(root: Path, ent: EnumEnt) -> TokenStream {
    let name = &ent.ident;
    let query_name = format_ident!("{}Query", name);
    let vis = &ent.vis;
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

    // We want this to be the total + 1 because we will include the enum
    //
    // NOTE: We assume that the fields are newtype from a filter done
    //       prior to this function call to limit to newtype enums
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let total_variants = enum_variants.len();
    let variant_types = enum_variants
        .into_iter()
        .map(|v| v.fields.iter().next().unwrap())
        .collect::<Vec<&Type>>();

    quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Begin building a new ent query
            pub fn query() -> #query_name #ty_generics #where_clause {
                <#query_name #ty_generics as ::std::default::Default>::default()
            }
        }

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
            fn default() -> Self {
                <Self as ::std::convert::From<#root::Query>>::from(
                    #root::Query::default().where_type(
                        #root::TypedPredicate::or(
                            {
                                let mut list = ::std::vec::Vec::with_capacity(#total_variants);
                                #(
                                    if let ::std::option::Option::Some(tys) =
                                        <#variant_types as #root::EntType>::wrapped_tys()
                                    {
                                        ::std::iter::Extend::<#root::TypedPredicate<_>>::extend(
                                            &mut list,
                                            ::std::iter::Iterator::map(
                                                ::std::iter::IntoIterator::into_iter(tys),
                                                |s| #root::TypedPredicate::equals(
                                                    ::std::string::ToString::to_string(s)
                                                ),
                                            ),
                                        );
                                    } else {
                                        list.push(#root::TypedPredicate::equals(
                                            ::std::string::ToString::to_string(
                                                <#variant_types as #root::EntType>::type_str()
                                            )
                                        ));
                                    }
                                )*
                                list
                            }
                        )
                    )
                )
            }
        }

        #[automatically_derived]
        impl #impl_generics #query_name #ty_generics #where_clause {
            #[doc = "Filters to return all ents where id passes the given predicate"]
            pub fn where_id(self, p: #root::TypedPredicate<#root::Id>) -> Self {
                Self(self.0.where_id(p), #(#default_phantoms),*)
            }

            #[doc = "Filters to return all ents where created timestamp passes the given predicate"]
            pub fn where_created(self, p: #root::TypedPredicate<::std::primitive::u64>) -> Self {
                Self(self.0.where_created(p), #(#default_phantoms),*)
            }

            #[doc = "Filters to return all ents where last updated timestamp passes the given predicate"]
            pub fn where_last_updated(self, p: #root::TypedPredicate<::std::primitive::u64>) -> Self {
                Self(self.0.where_last_updated(p), #(#default_phantoms),*)
            }

            #[doc = "Filters to return all ents where filed passes the given predicate"]
            pub fn where_field(self, name: &::std::primitive::str, p: #root::Predicate) -> Self {
                Self(self.0.where_field(name, p), #(#default_phantoms),*)
            }
        }

        #[automatically_derived]
        impl #impl_generics #root::EntQuery for #query_name #ty_generics #where_clause {
            type Output = ::std::vec::Vec<#name #ty_generics>;

            fn execute<D: #root::Database>(
                self,
                database: &D,
            ) -> #root::DatabaseResult<Self::Output> {
                ::std::result::Result::Ok(
                    ::std::iter::Iterator::collect(
                        ::std::iter::Iterator::filter_map(
                            ::std::iter::IntoIterator::into_iter(
                                database.find_all(self.0)?
                            ),
                            <#name #ty_generics as #root::EntWrapper>::wrap_ent,
                        )
                    )
                )
            }
        }
    }
}
