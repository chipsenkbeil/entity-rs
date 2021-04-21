#![forbid(unsafe_code)]

mod data;
mod derive;
mod utils;

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
