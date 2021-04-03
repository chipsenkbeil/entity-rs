use crate::{data::r#enum::Ent, utils};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn do_derive_ent_type(root: Path, ent: Ent) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let type_str_t = utils::make_type_str(name);

    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let wrapped_type_strs_t: Vec<TokenStream> = enum_variants
        .into_iter()
        .map(|v| {
            // NOTE: We assume that the fields are newtype from a filter done
            //       prior to this function call to limit to newtype enums
            let type_t = v.fields.iter().next().unwrap();

            quote! {
                match <#type_t as #root::EntType>::type_data() {
                    #root::EntTypeData::Concrete { ty } => {
                        let _ = tys.insert(ty);
                    },
                    #root::EntTypeData::Wrapper { ty: _, wrapped_tys } =>
                        ::std::iter::Extend::<&'static ::std::primitive::str>::extend(
                            &mut tys,
                            wrapped_tys,
                        ),
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl #impl_generics #root::EntType for #name #ty_generics #where_clause {
            fn type_data() -> #root::EntTypeData {
                #root::EntTypeData::Wrapper {
                    ty: #type_str_t,
                    wrapped_tys: {
                        let mut tys = ::std::collections::HashSet::new();
                        #(#wrapped_type_strs_t)*
                        tys
                    },
                }
            }
        }
    }
}
