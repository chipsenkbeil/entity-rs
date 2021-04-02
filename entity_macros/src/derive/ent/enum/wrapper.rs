use crate::data::r#enum::Ent;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn do_derive_ent_wrapper(root: Path, ent: Ent) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let enum_variants = ent.data.as_ref().take_enum().unwrap();

    let variant_conversions: Vec<TokenStream> = enum_variants
        .iter()
        .map(|v| {
            let name_t = &v.ident;

            // NOTE: We assume that the fields are newtype from a filter done
            //       prior to this function call to limit to newtype enums
            let type_t = v.fields.iter().next().unwrap();

            // If the variant is flagged with "wrap", then we treat it as an ent wrapper type
            if v.wrap.is_some() {
                quote! {
                    if <#type_t as #root::EntWrapper>::can_wrap_ent(
                        ::std::convert::AsRef::<#root::Ent>::as_ref(
                            &ent
                        )
                    ) {
                        return ::std::option::Option::Some(#name::#name_t(
                            ::std::option::Option::unwrap(
                                <#type_t as #root::EntWrapper>::wrap_ent(ent)
                            )
                        ))
                    }
                }
            // Otherwise, we assume that the variant is a base type and try to convert to it
            } else {
                quote! {
                    if let ::std::option::Option::Some(x) = ent.to_ent::<#type_t>() {
                        return ::std::option::Option::Some(#name::#name_t(x));
                    }
                }
            }
        })
        .collect();

    let variant_type_checks: Vec<TokenStream> = enum_variants
        .iter()
        .map(|v| {
            // NOTE: We assume that the fields are newtype from a filter done
            //       prior to this function call to limit to newtype enums
            let type_t = v.fields.iter().next().unwrap();

            // If the variant is flagged with "wrap", then we treat it as an ent wrapper type
            if v.wrap.is_some() {
                quote! {
                    if <#type_t as #root::EntWrapper>::can_wrap_ent(ent) {
                        return true;
                    }
                }
            // Otherwise, we assume that the variant is a base type and try to convert to it
            } else {
                quote! {
                    if let ::std::option::Option::Some(_) = ent.to_ent::<#type_t>() {
                        return true;
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #impl_generics #root::EntWrapper for #name #ty_generics #where_clause {
            fn wrap_ent(ent: ::std::boxed::Box<dyn #root::Ent>) -> ::std::option::Option<Self> {
                #(#variant_conversions)*
                ::std::option::Option::None
            }

            fn can_wrap_ent(ent: &dyn #root::Ent) -> ::std::primitive::bool {
                #(#variant_type_checks)*
                false
            }
        }
    }
}
