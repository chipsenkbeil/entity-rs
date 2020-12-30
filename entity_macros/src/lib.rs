mod attribute;
mod derive;
mod utils;

use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemStruct};

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);

/// Derives the Ent trait and additional typed functionality
///
/// ```
/// use entity::{Ent, Id, Database};
///
/// /// Define an entity and derive all associated ent functionality
/// ///
/// /// The entity must also implement clone as this is a requirement of
/// /// the Ent trait
/// ///
/// /// If using serde, this struct will need to implement serialize and
/// /// deserialize itself AND include the attribute ent(typetag)
/// #[derive(Clone, Ent)]
/// pub struct PageEnt {
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the ent's id
///     #[ent(id)]
///     id: Id,
///
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the database. Must be an option!
///     ///
///     /// If using serde, this field will need to be skipped via serde(skip)
///     /// as it will not be serialized and, when deserializing, will be
///     /// filled in with the database automatically
///     #[ent(database)]
///     database: Option<Box<dyn Database>>,
///
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the timestamp of when the ent was created
///     #[ent(created)]
///     created: u64,
///
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the timestamp of when the ent was last updated
///     #[ent(last_updated)]
///     last_updated: u64,
///
///     /// A public ent field that is indexed, meaning that searches for this
///     /// ent by its title should be faster, but this will also take up
///     /// more space in the database
///     #[ent(field(indexed))]
///     title: String,
///
///     /// A public ent field that is not indexed
///     #[ent(field)]
///     url: String,
///
///     /// An edge out to a ContentEnt that is shallowly connected, meaning
///     /// that when this ent is deleted, the ent connected by this edge
///     /// will remove this ent if it is reversely-connected
///     #[ent(edge(policy = "shallow", type = "ContentEnt"))]
///     header: Id,
///
///     /// An optional edge out to a ContentEnt that is deeply connected,
///     /// meaning that when this ent is deleted, the ent connected by this
///     /// edge will also be deleted
///     #[ent(edge(policy = "deep", type = "ContentEnt"))]
///     subheader: Option<Id>,
///
///     /// An edge out to zero or more ContentEnt, defaulting to doing
///     /// nothing special when this ent is deleted
///     #[ent(edge(type = "ContentEnt"))]
///     paragraphs: Vec<Id>,
/// }
///
/// #[derive(Clone, Ent)]
/// pub struct ContentEnt {
///     #[ent(id)]
///     id: Id,
///
///     #[ent(database)]
///     database: Option<Box<dyn Database>>,
///
///     #[ent(created)]
///     created: u64,
///
///     #[ent(last_updated)]
///     last_updated: u64,
///
///     #[ent(field)]
///     text: String,
/// }
/// ```
#[proc_macro_derive(Ent, attributes(ent))]
pub fn derive_ent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = utils::root()
        .and_then(|root| derive::do_derive_ent(root, input))
        .unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Value, attributes(value))]
pub fn derive_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = utils::root()
        .and_then(|root| derive::do_derive_value(root, input))
        .unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}

/// Injects elements needed for an ent to be derived.
///
/// ```
/// use entity::{simple_ent};
///
/// #[simple_ent]
/// pub struct MyEnt {
///     name: String,
///     value: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn simple_ent(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let expanded = utils::root()
        .and_then(|root| attribute::do_simple_ent(root, args, input))
        .unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}
