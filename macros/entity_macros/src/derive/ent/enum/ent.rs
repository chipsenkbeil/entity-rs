use crate::utils;
use entity_macros_data::EnumEnt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

pub fn do_derive_ent(root: Path, ent: EnumEnt) -> darling::Result<TokenStream> {
    let name = &ent.ident;
    let generics = &ent.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let enum_variants = ent.data.as_ref().take_enum().unwrap();
    let variant_names: Vec<&Ident> = enum_variants.iter().map(|v| &v.ident).collect();

    let typetag_root = utils::typetag_crate()?;
    let typetag_t = quote!(#[#typetag_root::serde]);

    Ok(quote! {
        #typetag_t
        #[automatically_derived]
        impl #impl_generics #root::Ent for #name #ty_generics #where_clause {
            fn id(&self) -> #root::Id {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::id(x)),*
                }
            }

            fn set_id(&mut self, id: #root::Id) {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::set_id(x, id)),*
                }
            }

            fn r#type(&self) -> &::std::primitive::str {
                ::std::concat!(
                    ::std::module_path!(),
                    "::",
                    ::std::stringify!(#name),
                )
            }

            fn created(&self) -> ::std::primitive::u64 {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::created(x)),*
                }
            }

            fn last_updated(&self) -> ::std::primitive::u64 {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::last_updated(x)),*
                }
            }

            fn mark_updated(&mut self) -> ::std::result::Result<(), #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::mark_updated(x)),*
                }
            }

            fn field_definitions(&self) -> ::std::vec::Vec<#root::FieldDefinition> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::field_definitions(x)),*
                }
            }

            fn field(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::Value> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::field(x, name)),*
                }
            }

            fn update_field(
                &mut self,
                name: &::std::primitive::str,
                value: #root::Value,
            ) -> ::std::result::Result<#root::Value, #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::update_field(x, name, value)),*
                }
            }

            fn edge_definitions(&self) -> ::std::vec::Vec<#root::EdgeDefinition> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::edge_definitions(x)),*
                }
            }

            fn edge(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::EdgeValue> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::edge(x, name)),*
                }
            }

            fn update_edge(
                &mut self,
                name: &::std::primitive::str,
                value: #root::EdgeValue,
            ) -> ::std::result::Result<#root::EdgeValue, #root::EntMutationError> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::update_edge(x, name, value)),*
                }
            }

            fn connect(&mut self, database: #root::WeakDatabaseRc) {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::connect(x, database)),*
                }
            }

            fn disconnect(&mut self) {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::disconnect(x)),*
                }
            }

            fn is_connected(&self) -> ::std::primitive::bool {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::is_connected(x)),*
                }
            }

            fn load_edge(
                &self,
                name: &::std::primitive::str,
            ) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::Ent>>> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::load_edge(x, name)),*
                }
            }

            fn clear_cache(&mut self) {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::clear_cache(x)),*
                }
            }

            fn refresh(&mut self) -> #root::DatabaseResult<()> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::refresh(x)),*
                }
            }

            fn commit(&mut self) -> #root::DatabaseResult<()> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::commit(x)),*
                }
            }

            fn remove(&self) -> #root::DatabaseResult<::std::primitive::bool> {
                match self {
                    #(Self::#variant_names(x) => #root::Ent::remove(x)),*
                }
            }
        }
    })
}
