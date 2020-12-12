use crate::derive::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, FieldsNamed, Generics, Ident};

pub fn make(
    root: &TokenStream,
    name: &Ident,
    generics: &Generics,
    fields: &FieldsNamed,
) -> TokenStream {
    let field_names: Vec<Ident> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap().clone())
        .collect();
    let temp_field_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("tmp_{}", name))
        .collect();
    let converted_values: Vec<Expr> = temp_field_names
        .iter()
        .zip(fields.named.iter().map(|f| &f.ty))
        .map(|(name, ty)| utils::convert_from_value(name, ty))
        .collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics> for #root::Value #where_clause {
            fn from(x: #name) -> Self {
                let mut map = ::std::collections::HashMap::new();
                #(
                    map.insert(
                        ::std::string::String::from(stringify!(#field_names)),
                        #root::Value::from(x.#field_names),
                    );
                )*
                Self::Map(map)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::Value> for #name #ty_generics #where_clause {
            type Error = &'static str;

            fn try_from(value: #root::Value) -> ::std::result::Result<Self, Self::Error> {
                let mut map = match value {
                    #root::Value::Map(x) => x,
                    _ => return ::std::result::Result::Err(concat!(
                        "Only Value::Map can be converted to ",
                        stringify!(#name),
                    )),
                };

                ::std::result::Result::Ok(Self {
                    #(
                        #field_names: {
                            let #temp_field_names = map.remove(
                                stringify!(#field_names)
                            ).ok_or(
                                concat!("Missing field ", stringify!(#field_names))
                            )?;

                            #converted_values?
                        }
                    ),*
                })
            }
        }
    }
}
