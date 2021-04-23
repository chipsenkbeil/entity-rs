mod ent_filter;
mod ent_object;

use crate::data::GqlEnt;
use darling::FromDeriveInput;
use entity_macros_data::StructEnt;
use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Path};

pub fn do_derive_ent_object(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            ent_object::do_derive_ent_object(root, StructEnt::from_derive_input(&input)?)
        }
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_filter(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => ent_filter::do_derive_ent_filter(
            root,
            StructEnt::from_derive_input(&input)?,
            GqlEnt::from_derive_input(&input)?,
        ),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}
