use crate::data::r#enum::Ent;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

pub fn do_derive_ent_wrapper(root: Path, ent: Ent) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let variant_names: Vec<&Ident> = enum_variants.iter().map(|v| &v.ident).collect();

    // NOTE: We assume that the fields are newtype from a filter done
    //       prior to this function call to limit to newtype enums
    let variant_types = enum_variants
        .into_iter()
        .map(|v| v.fields.iter().next().unwrap())
        .collect::<Vec<&Type>>();

    quote! {
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
    }
}
