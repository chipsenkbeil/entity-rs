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
        impl #impl_generics #root::ValueLike for #name #ty_generics #where_clause {
            fn into_value(self) -> #root::Value {
                let mut list = ::std::vec::Vec::new();
                #(
                    list.push(
                        <#field_types as #root::ValueLike>::into_value(self.#field_names),
                    );
                )*
                #root::Value::List(list)
            }

            fn try_from_value(value: #root::Value) -> ::std::result::Result<Self, #root::Value> {
                let mut list = match value {
                    #root::Value::List(x) if x.len() == #lit_cnt => x,
                    x => return ::std::result::Result::Err(x),
                };

                // Validate that each of our fields exists and can be converted
                // into the specific type
                #({
                    let #temp_field_names = list.remove(#field_names);
                    let result = #converted_values;

                    // We re-add it back to the map, but if it fails to
                    // convert then we return the list as an error
                    match result {
                        ::std::result::Result::Ok(x) => {
                            list.insert(
                                #field_names,
                                #root::ValueLike::into_value(x),
                            );
                        },
                        ::std::result::Result::Err(value) => {
                            list.insert(#field_names, value);
                            return ::std::result::Result::Err(#root::Value::List(list));
                        }
                    }
                })*

                let mut list_it = ::std::iter::IntoIterator::into_iter(list);

                ::std::result::Result::Ok(Self(
                    #(
                        {
                            let #temp_field_names = ::std::iter::Iterator::next(&mut list_it).unwrap();
                            #converted_values.unwrap()
                        }
                    ),*
                ))
            }
        }
    }
}
