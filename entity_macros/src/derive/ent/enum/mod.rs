mod data;

use crate::utils;
use darling::FromDeriveInput;
use data::Ent;
use heck::ShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, spanned::Spanned, DeriveInput, Expr, Ident, Path, Type};

pub fn do_derive_ent(root: Path, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ent = data::Ent::from_derive_input(&input)
        .map_err(|e| syn::Error::new(input.span(), e.to_string()))?;

    let (const_type_name, const_t) = impl_const(&root, &ent);
    let query_t = if ent.no_query {
        quote! {}
    } else {
        impl_query(&root, &ent)?
    };
    let ent_wrapper_t = impl_ent_wrapper(&root, &ent)?;
    let ent_t = impl_ent(&root, &ent, &const_type_name)?;

    Ok(quote! {
        #const_t
        #query_t
        #ent_wrapper_t
        #ent_t
    })
}

fn impl_const(_root: &Path, ent: &Ent) -> (Ident, TokenStream) {
    let name = &ent.ident;
    let vis = &ent.vis;
    let const_type_name = format_ident!("{}_TYPE", name.to_string().to_shouty_snake_case());
    let const_t = quote! {
        #vis const #const_type_name: &::std::primitive::str = ::std::concat!(
            ::std::module_path!(), "::", ::std::stringify!(#name),
        );
    };
    (const_type_name, const_t)
}

fn impl_query(root: &Path, ent: &Ent) -> Result<TokenStream, syn::Error> {
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
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let total_variants = enum_variants.len();
    let variant_types = enum_variants
        .into_iter()
        .map(|v| {
            if v.fields.is_newtype() {
                Ok(v.fields.iter().next().unwrap())
            } else {
                Err(syn::Error::new(v.ident.span(), "Variant must be newtype"))
            }
        })
        .collect::<Result<Vec<&Type>, syn::Error>>()?;

    Ok(quote! {
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
                                    list.push(#root::TypedPredicate::equals(
                                        ::std::string::ToString::to_string(<#variant_types as #root::EntType>::type_str())
                                    ));
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

            #[doc = "Executes query against the given database"]
            pub fn execute<__entity_D: #root::Database>(
                self,
                database: &__entity_D,
            ) -> #root::DatabaseResult<::std::vec::Vec<#name #ty_generics>> {
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
    })
}

fn impl_ent_wrapper(root: &Path, ent: &Ent) -> Result<TokenStream, syn::Error> {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let variant_names: Vec<&Ident> = enum_variants.iter().map(|v| &v.ident).collect();
    let variant_types = enum_variants
        .into_iter()
        .map(|v| {
            if v.fields.is_newtype() {
                Ok(v.fields.iter().next().unwrap())
            } else {
                Err(syn::Error::new(v.ident.span(), "Variant must be newtype"))
            }
        })
        .collect::<Result<Vec<&Type>, syn::Error>>()?;

    Ok(quote! {
        impl #impl_generics #root::EntWrapper for #name #ty_generics #where_clause {
            fn wrap_ent(ent: ::std::boxed::Box<dyn #root::Ent>) -> ::std::option::Option<Self> {
                #(
                    if let ::std::option::Option::Some(x) = ent.to_ent::<#variant_types>() {
                        return ::std::option::Option::Some(#name::#variant_names(x));
                    }
                )*

                ::std::option::Option::None
            }
        }
    })
}

fn impl_ent(root: &Path, ent: &Ent, const_type_name: &Ident) -> Result<TokenStream, syn::Error> {
    let name = &ent.ident;
    let generics = &ent.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let variant_names: Vec<&Ident> = enum_variants.iter().map(|v| &v.ident).collect();

    let typetag_t = if ent.typetag {
        let typetag_root = utils::typetag_crate()?;
        quote! { #[#typetag_root::serde] }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #root::EntType for #name #ty_generics #where_clause {
            fn type_str() -> &'static ::std::primitive::str {
                #const_type_name
            }
        }

        #typetag_t
        #[automatically_derived]
        impl #impl_generics #root::Ent for #name #ty_generics #where_clause {
            fn id(&self) -> #root::Id {
                match self {
                    #(Self::#variant_names(x) => x.id()),*
                }
            }

            fn set_id(&mut self, id: #root::Id) {
                match self {
                    #(Self::#variant_names(x) => x.set_id(id)),*
                }
            }

            fn r#type(&self) -> &::std::primitive::str {
                #const_type_name
            }

            fn created(&self) -> ::std::primitive::u64 {
                match self {
                    #(Self::#variant_names(x) => x.created()),*
                }
            }

            fn last_updated(&self) -> ::std::primitive::u64 {
                match self {
                    #(Self::#variant_names(x) => x.last_updated()),*
                }
            }

            fn mark_updated(&mut self) -> ::std::result::Result<(), #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => x.mark_updated()),*
                }
            }

            fn field_definitions(&self) -> ::std::vec::Vec<#root::FieldDefinition> {
                match self {
                    #(Self::#variant_names(x) => x.field_definitions()),*
                }
            }

            fn field(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::Value> {
                match self {
                    #(Self::#variant_names(x) => x.field(name)),*
                }
            }

            fn update_field(
                &mut self,
                name: &::std::primitive::str,
                value: #root::Value,
            ) -> ::std::result::Result<#root::Value, #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => x.update_field(name, value)),*
                }
            }

            fn edge_definitions(&self) -> ::std::vec::Vec<#root::EdgeDefinition> {
                match self {
                    #(Self::#variant_names(x) => x.edge_definitions()),*
                }
            }

            fn edge(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::EdgeValue> {
                match self {
                    #(Self::#variant_names(x) => x.edge(name)),*
                }
            }

            fn update_edge(
                &mut self,
                name: &::std::primitive::str,
                value: #root::EdgeValue,
            ) -> ::std::result::Result<#root::EdgeValue, #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => x.update_edge(name, value)),*
                }
            }

            fn connect(&mut self, database: #root::WeakDatabaseRc) {
                match self {
                    #(Self::#variant_names(x) => x.connect(database)),*
                }
            }

            fn disconnect(&mut self) {
                match self {
                    #(Self::#variant_names(x) => x.disconnect()),*
                }
            }

            fn is_connected(&self) -> ::std::primitive::bool {
                match self {
                    #(Self::#variant_names(x) => x.is_connected()),*
                }
            }

            fn load_edge(
                &self,
                name: &::std::primitive::str,
            ) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::Ent>>> {
                match self {
                    #(Self::#variant_names(x) => x.load_edge(name)),*
                }
            }

            fn refresh(&mut self) -> #root::DatabaseResult<()> {
                match self {
                    #(Self::#variant_names(x) => x.refresh()),*
                }
            }

            fn commit(&mut self) -> #root::DatabaseResult<()> {
                match self {
                    #(Self::#variant_names(x) => x.commit()),*
                }
            }

            fn remove(&self) -> #root::DatabaseResult<::std::primitive::bool> {
                match self {
                    #(Self::#variant_names(x) => x.remove()),*
                }
            }
        }
    })
}
