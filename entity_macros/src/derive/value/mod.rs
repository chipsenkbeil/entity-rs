mod named;
mod unit;
mod unnamed;

use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Fields, Path};

pub fn do_derive_value(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;

    match &input.data {
        Data::Struct(x) => match &x.fields {
            Fields::Named(x) => Ok(named::make(&root, name, generics, x)),
            Fields::Unnamed(x) => Ok(unnamed::make(&root, name, generics, x)),
            Fields::Unit => Ok(unit::make(&root, name, generics)),
        },
        Data::Enum(_) => Err(darling::Error::custom("Enums are unsupported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are unsupported").with_span(&input)),
    }
}
