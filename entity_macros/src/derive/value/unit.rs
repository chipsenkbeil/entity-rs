use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident};

pub fn make(root: &TokenStream, name: &Ident, generics: &Generics) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics> for #root::Value #where_clause {
            fn from(x: #name) -> Self {
                Self::from(#root::PrimitiveValue::Unit)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::Value> for #name #ty_generics #where_clause {
            type Error = &'static str;

            fn try_from(x: #root::Value) -> ::std::result::Result<Self, Self::Error> {
                match x {
                    #root::Value::Primitive(#root::PrimitiveValue::Unit) =>
                        ::std::result::Result::Ok(Self),
                    _ => ::std::result::Result::Err("Value is not unit"),
                }
            }
        }
    }
}
