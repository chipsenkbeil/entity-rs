mod r#enum;
mod r#struct;

use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Path};

pub fn do_derive_ent(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => r#struct::do_derive_ent(root, input),
        Data::Enum(_) => r#enum::do_derive_ent(root, input),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}
