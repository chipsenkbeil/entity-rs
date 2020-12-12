use crate::derive::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, FieldsUnnamed, Generics, Ident, Index, LitInt};

pub fn make(
    root: &TokenStream,
    name: &Ident,
    generics: &Generics,
    fields: &FieldsUnnamed,
) -> TokenStream {
    let field_names: Vec<Index> = (0..fields.unnamed.len()).map(Index::from).collect();
    let temp_field_names: Vec<Ident> = (0..fields.unnamed.len())
        .map(|name| format_ident!("tmp_{}", name))
        .collect();
    let converted_values: Vec<Expr> = temp_field_names
        .iter()
        .zip(fields.unnamed.iter().map(|f| &f.ty))
        .map(|(name, ty)| utils::convert_from_value(name, ty))
        .collect();
    let cnt = temp_field_names.len();
    let lit_cnt: LitInt = parse_quote!(#cnt);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics> for #root::Value #where_clause {
            fn from(x: #name) -> Self {
                let mut list = ::std::vec::Vec::new();
                #(
                    list.push(
                        #root::Value::from(x.#field_names),
                    );
                )*
                Self::List(list)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::Value> for #name #ty_generics #where_clause {
            type Error = &'static str;

            fn try_from(value: #root::Value) -> ::std::result::Result<Self, Self::Error> {
                let list = match value {
                    #root::Value::List(x) if x.len() == #lit_cnt => x,
                    #root::Value::List(_) => return ::std::result::Result::Err(concat!(
                        "Only Value::List of len ",
                        stringify!(#lit_cnt),
                        " can be converted to ",
                        stringify!(#name),
                    )),
                    _ => return ::std::result::Result::Err(concat!(
                        "Only Value::List can be converted to ",
                        stringify!(#name),
                    )),
                };

                let mut list_it = list.into_iter();

                ::std::result::Result::Ok(Self(
                    #(
                        {
                            let #temp_field_names = list_it.next().unwrap();
                            #converted_values?
                        }
                    ),*
                ))
            }
        }
    }
}
