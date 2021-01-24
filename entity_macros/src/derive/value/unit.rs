use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident, Path};

pub fn make(root: &Path, name: &Ident, generics: &Generics) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics #root::ValueLike for #name #ty_generics #where_clause {
            fn into_value(self) -> #root::Value {
                #root::ValueLike::into_value(#root::Primitive::Unit)
            }

            fn try_from_value(value: #root::Value) -> ::std::result::Result<Self, #root::Value> {
                match value {
                    #root::Value::Primitive(#root::Primitive::Unit) =>
                        ::std::result::Result::Ok(Self),
                    x => ::std::result::Result::Err(x),
                }
            }
        }
    }
}
