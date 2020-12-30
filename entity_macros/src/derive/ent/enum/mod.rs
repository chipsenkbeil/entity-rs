mod data;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Path};

pub fn do_derive_ent(_root: Path, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let _ent = data::Ent::from_derive_input(&input)
        .map_err(|e| syn::Error::new(input.span(), e.to_string()))?;
    // let name = &ent.ident;
    // let vis = &ent.vis;
    // let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    Ok(quote! {})
    // Ok(quote! {
    //     #ent_t
    //     #query_t
    // })
}
