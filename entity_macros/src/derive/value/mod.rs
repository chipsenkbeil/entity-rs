mod named;
mod unit;
mod unnamed;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path};

pub fn do_derive_value_like(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;

    match &input.data {
        Data::Struct(x) => match &x.fields {
            Fields::Named(x) => Ok(named::make(&root, name, generics, x)),
            Fields::Unnamed(x) => Ok(unnamed::make(&root, name, generics, x)),
            Fields::Unit => Ok(unit::make(&root, name, generics)),
        },
        Data::Enum(_) => Err(darling::Error::custom("Enums are unsupported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are unsupported").with_span(&input)),
    }
}

pub fn do_derive_into_value(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Struct(_) => Ok(quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#name #ty_generics> for #root::Value #where_clause {
                fn from(x: #name #ty_generics) -> Self {
                    <#name #ty_generics as #root::ValueLike>::into_value(x)
                }
            }
        }),
        Data::Enum(_) => Err(darling::Error::custom("Enums are unsupported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are unsupported").with_span(&input)),
    }
}

pub fn do_derive_try_from_value(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Struct(_) => Ok(quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#root::Value> for #name #ty_generics #where_clause {
                type Error = #root::Value;

                fn try_from(x: #root::Value) -> ::std::result::Result<Self, Self::Error> {
                    <#name #ty_generics as #root::ValueLike>::try_from_value(x)
                }
            }
        }),
        Data::Enum(_) => Err(darling::Error::custom("Enums are unsupported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are unsupported").with_span(&input)),
    }
}
