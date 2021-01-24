use crate::utils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, FieldsNamed, Generics, Ident, Path, Type};

pub fn make(root: &Path, name: &Ident, generics: &Generics, fields: &FieldsNamed) -> TokenStream {
    let field_names: Vec<Ident> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap().clone())
        .collect();
    let field_types: Vec<&Type> = fields.named.iter().map(|f| &f.ty).collect();
    let temp_field_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("tmp_{}", name))
        .collect();
    let converted_values: Vec<Expr> = temp_field_names
        .iter()
        .zip(fields.named.iter().map(|f| &f.ty))
        .map(|(name, ty)| utils::convert_from_value(root, name, ty))
        .collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics #root::ValueLike for #name #ty_generics #where_clause {
            fn into_value(self) -> #root::Value {
                let mut map = ::std::collections::HashMap::new();
                #(
                    map.insert(
                        ::std::string::ToString::to_string(::std::stringify!(#field_names)),
                        <#field_types as #root::ValueLike>::into_value(self.#field_names),
                    );
                )*
                #root::Value::Map(map)
            }

            fn try_from_value(value: #root::Value) -> ::std::result::Result<Self, #root::Value> {
                let mut map = match value {
                    #root::Value::Map(x) => x,
                    x => return ::std::result::Result::Err(x),
                };

                // Validate that each of our fields exists and can be converted
                // into the specific type
                #({
                    match map.remove(::std::stringify!(#field_names)) {
                        ::std::option::Option::Some(#temp_field_names) => {
                            let result = #converted_values;

                            // If the field exists, we see if it can be converted
                            // into the right underlying type. Either way, we
                            // re-add it back to the map, but if it fails to
                            // convert then we return the map as an error
                            match result {
                                ::std::result::Result::Ok(x) => {
                                    map.insert(
                                        ::std::string::ToString::to_string(::std::stringify!(#field_names)),
                                        #root::ValueLike::into_value(x),
                                    );
                                },
                                ::std::result::Result::Err(value) => {
                                    map.insert(
                                        ::std::string::ToString::to_string(::std::stringify!(#field_names)),
                                        value,
                                    );
                                    return ::std::result::Result::Err(#root::Value::Map(map));
                                }
                            }
                        }
                        ::std::option::Option::None => return ::std::result::Result::Err(
                            #root::Value::Map(map),
                        ),
                    }
                })*

                ::std::result::Result::Ok(Self {
                    #(
                        #field_names: {
                            let #temp_field_names = map.remove(
                                ::std::stringify!(#field_names)
                            ).unwrap();

                            #converted_values.unwrap()
                        }
                    ),*
                })
            }
        }
    }
}
