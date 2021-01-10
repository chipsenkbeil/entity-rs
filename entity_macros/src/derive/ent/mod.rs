mod r#enum;
mod r#struct;

use crate::data::{r#enum::Ent as EnumEnt, r#struct::Ent as StructEnt};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Path};

pub fn do_derive_ent(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => r#struct::do_derive_ent(root, StructEnt::from_derive_input(&input)?),
        Data::Enum(_) => r#enum::do_derive_ent(root, EnumEnt::from_derive_input(&input)?),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_wrapper(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            Err(darling::Error::custom("Structs are not supported").with_span(&input))
        }
        Data::Enum(_) => r#enum::do_derive_ent_wrapper(root, EnumEnt::from_derive_input(&input)?),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_builder(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            r#struct::do_derive_ent_builder(root, StructEnt::from_derive_input(&input)?)
        }
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_loader(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(r#struct::do_derive_ent_loader(
            root,
            StructEnt::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_debug(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(r#struct::do_derive_ent_debug(
            root,
            StructEnt::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Err(darling::Error::custom(
            "Enums are not supported, derive Debug instead",
        )
        .with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_query(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            r#struct::do_derive_ent_query(root, StructEnt::from_derive_input(&input)?)
        }
        Data::Enum(_) => r#enum::do_derive_ent_query(root, EnumEnt::from_derive_input(&input)?),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_type(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(r#struct::do_derive_ent_type(
            root,
            StructEnt::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Ok(r#enum::do_derive_ent_type(
            root,
            EnumEnt::from_derive_input(&input)?,
        )),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_typed_edges(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            r#struct::do_derive_ent_typed_edges(root, StructEnt::from_derive_input(&input)?)
        }
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_ent_typed_fields(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(r#struct::do_derive_ent_typed_fields(
            root,
            StructEnt::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_async_graphql_ent(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => {
            r#struct::do_derive_async_graphql_ent(root, StructEnt::from_derive_input(&input)?)
        }
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

pub fn do_derive_async_graphql_ent_filter(
    root: Path,
    input: DeriveInput,
) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => r#struct::do_derive_async_graphql_ent_filter(
            root,
            StructEnt::from_derive_input(&input)?,
        ),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}
