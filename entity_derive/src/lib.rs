use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::path::Path;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Fields, Ident, ItemStruct, Type, Visibility};

/// Transforms pseudo-struct syntax into an ent representation
///
/// ```
/// use entity::{Id, Database};
///
/// /// Define an entity and derive all associated ent functionality
/// ///
/// /// The entity must also implement clone as this is a requirement of
/// /// the IEnt trait
/// #[derive(Clone, Ent)]
/// pub struct PageEnt {
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the ent's id
///     #[ent(id)]
///     id: Id,
///
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the database. Must be an option!
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
///     #[ent(edge(shallow, type = "ContentEnt"))]
///     header: Id,
///
///     /// An optional edge out to a ContentEnt that is deeply connected,
///     /// meaning that when this ent is deleted, the ent connected by this
///     /// edge will also be deleted
///     #[ent(edge(deep, type = "ContentEnt"))]
///     subheader: Option<Id>,
///
///     /// An edge out to zero or more ContentEnt, defaulting to doing
///     /// nothing special when this ent is deleted
///     #[ent(edge(type = "ContentEnt"))]
///     paragraphs: Vec<Id>,
/// }
/// ```
#[proc_macro_derive(Ent)]
pub fn derive_ent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let root = quote! { ::entity };
    let expanded = impl_ent(root, input).unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}

#[inline]
fn impl_ent(root: TokenStream, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = input.ident;
    let vis = input.vis;
    let const_type_name = format_ident!("{}Type", name);

    let typetag_t = if cfg!(feature = "typetag") {
        quote! { #[#root::vendor::typetag::serde] }
    } else {
        quote! {}
    };

    // Build the output, possibly using quasi-quotation
    Ok(quote! {
        #vis const #const_type_name: &str = concat!(module_path!(), "::", stringify!(#name));

        /// CHIP CHIP CHIP
        /// Alongside the trait impl, we also want to add explicit
        /// methods to load ents from edges using specific types
        impl #name {
            pub fn load_header(&self) -> #root::DatabaseResult<ContentEnt> {
                todo!()
            }

            pub fn load_subheader(&self) -> #root::DatabaseResult<::std::vec::Option<ContentEnt>> {
                todo!()
            }

            pub fn load_paragraphs(&self) -> #root::DatabaseResult<::std::vec::Vec<ContentEnt>> {
                todo!()
            }
        }

        #typetag_t
        impl #root::IEnt for #name {
            fn id(&self) -> #root::Id {
                todo!()
            }

            fn set_id(&mut self, id: #root::Id) {
                todo!();
            }

            fn r#type(&self) -> &str {
                #const_type_name
            }

            fn created(&self) -> u64 {
                todo!()
            }

            fn last_updated(&self) -> u64 {
                todo!()
            }

            fn field_names(&self) -> ::std::vec::Vec<::std::string::String> {
                todo!()
            }

            fn field(&self, name: &str) -> ::std::option::Option<#root::Value> {
                todo!()
            }

            fn update_field(&mut self, name: &str, value: #root::Value) -> #root::EntMutationError {
                todo!()
            }

            fn edge_names(&self) -> ::std::vec::Vec<::std::string::String> {
                todo!()
            }

            fn edge(&self, name: &str) -> ::std::vec::Vec<#root::EdgeValue> {
                todo!()
            }

            fn update_edge(&self, name: &str, value: #root::EdgeValue) -> #root::EntMutationError {
                todo!()
            }

            fn connect(&mut self, database: ::std::boxed::Box<dyn #root::Database>) {
                todo!()
            }

            fn disconnect(&mut self) {
                todo!()
            }

            fn is_connected(&self) -> bool {
                todo!()
            }

            fn load_edge(&self, name: &str) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::IEnt>>> {
                todo!()
            }

            fn refresh(&mut self) -> #root::DatabaseResult<()> {
                todo!()
            }

            fn commit(&mut self) -> #root::DatabaseResult<()> {
                todo!()
            }

            fn delete(self) -> #root::DatabaseResult<bool> {
                todo!()
            }
        }
    })
}
