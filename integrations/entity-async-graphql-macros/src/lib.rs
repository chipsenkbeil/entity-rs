#![forbid(unsafe_code)]

mod attribute;
mod data;
mod derive;
mod utils;

use syn::{parse_macro_input, AttributeArgs, DeriveInput};

/// Special wrapper to derive an async-graphql object based on the ent
#[proc_macro_derive(EntObject, attributes(ent))]
pub fn derive_ent_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::do_derive(derive::do_derive_ent_object)(input)
}

/// Special wrapper to derive an async-graphql filter based on the ent
#[proc_macro_derive(EntFilter, attributes(ent))]
pub fn derive_ent_filter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::do_derive(derive::do_derive_ent_filter)(input)
}

/// Injects elements needed for an ent to be derived as a GraphQL entity.
///
/// ```
/// use entity_async_graphql::gql_ent;
///
/// #[gql_ent]
/// pub struct MyEnt {
///     name: String,
///     value: u32,
///
///     #[ent(field(computed = "123"))]
///     computed_value: u8,
///
///     #[ent(edge)]
///     other: MyEnt,
/// }
/// ```
#[proc_macro_attribute]
pub fn gql_ent(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = utils::entity_crate()
        .and_then(|root| attribute::do_gql_ent(root, args, input))
        .unwrap_or_else(|x| x.write_errors());

    proc_macro::TokenStream::from(expanded)
}
