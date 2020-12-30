mod r#enum;
mod r#struct;

use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Data, DeriveInput, Path};

pub fn do_derive_ent(root: Path, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    match &input.data {
        Data::Struct(_) => r#struct::do_derive_ent(root, input),
        Data::Enum(_) => r#enum::do_derive_ent(root, input),
        Data::Union(_) => Err(syn::Error::new(input.span(), "Unions are not supported")),
    }
}
