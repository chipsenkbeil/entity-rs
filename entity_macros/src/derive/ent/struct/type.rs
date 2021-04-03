use crate::{data::r#struct::Ent, utils};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn do_derive_ent_type(root: Path, ent: Ent) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let type_str_t = utils::make_type_str(name);

    quote! {
        #[automatically_derived]
        impl #impl_generics #root::EntType for #name #ty_generics #where_clause {
            fn type_data() -> #root::EntTypeData {
                #root::EntTypeData::Concrete { ty: #type_str_t }
            }
        }
    }
}
