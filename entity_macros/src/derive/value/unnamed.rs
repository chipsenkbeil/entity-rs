use crate::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, FieldsUnnamed, Generics, Ident, Index, LitInt, Path, Type};

pub fn make(root: &Path, name: &Ident, generics: &Generics, fields: &FieldsUnnamed) -> TokenStream {
    let field_names: Vec<Index> = (0..fields.unnamed.len()).map(Index::from).collect();
    let field_types: Vec<&Type> = fields.unnamed.iter().map(|f| &f.ty).collect();
    let temp_field_names: Vec<Ident> = (0..fields.unnamed.len())
        .map(|name| format_ident!("tmp_{}", name))
        .collect();
    let converted_values: Vec<Expr> = temp_field_names
        .iter()
        .zip(fields.unnamed.iter().map(|f| &f.ty))
        .map(|(name, ty)| utils::convert_from_value(root, name, ty))
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
                        <#root::Value as ::std::convert::From<#field_types>>::from(x.#field_names),
                    );
                )*
                Self::List(list)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::Value> for #name #ty_generics #where_clause {
            type Error = &'static ::std::primitive::str;

            fn try_from(value: #root::Value) -> ::std::result::Result<Self, Self::Error> {
                let list = match value {
                    #root::Value::List(x) if x.len() == #lit_cnt => x,
                    #root::Value::List(_) => return ::std::result::Result::Err(::std::concat!(
                        "Only Value::List of len ",
                        ::std::stringify!(#lit_cnt),
                        " can be converted to ",
                        ::std::stringify!(#name),
                    )),
                    _ => return ::std::result::Result::Err(::std::concat!(
                        "Only Value::List can be converted to ",
                        ::std::stringify!(#name),
                    )),
                };

                let mut list_it = ::std::iter::IntoIterator::into_iter(list);

                ::std::result::Result::Ok(Self(
                    #(
                        {
                            let #temp_field_names = ::std::iter::Iterator::next(&mut list_it).unwrap();
                            #converted_values?
                        }
                    ),*
                ))
            }
        }
    }
}
