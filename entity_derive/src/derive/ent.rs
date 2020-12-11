use super::EntInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub(crate) fn impl_ent(
    root: &TokenStream,
    name: &Ident,
    ent_info: &EntInfo,
    const_type_name: &Ident,
    include_typetag: bool,
) -> TokenStream {
    let ident_id = &ent_info.id;
    let ident_database = &ent_info.database;
    let ident_created = &ent_info.created;
    let ident_last_updated = &ent_info.last_updated;
    let fields = &ent_info.fields;
    let edges = &ent_info.edges;

    let field_names: Vec<Ident> = fields.iter().map(|f| f.name.clone()).collect();
    let field_types: Vec<Type> = fields.iter().map(|f| f.ty.clone()).collect();
    let edge_names: Vec<Ident> = edges.iter().map(|e| e.name.clone()).collect();
    let edge_types: Vec<Type> = edges.iter().map(|e| e.ty.clone()).collect();

    // If we have the attribute ent(typetag) on our struct, we will add a
    // new attribute of #[typetag::serde] onto our impl of IEnt
    let typetag_t: TokenStream = if include_typetag {
        quote! { #[::typetag::serde] }
    } else {
        quote! {}
    };

    quote! {
        #typetag_t
        impl #root::IEnt for #name {
            fn id(&self) -> #root::Id {
                self.#ident_id
            }

            fn set_id(&mut self, id: #root::Id) {
                self.#ident_id = id;
            }

            fn r#type(&self) -> &str {
                #const_type_name
            }

            fn created(&self) -> u64 {
                self.#ident_created
            }

            fn last_updated(&self) -> u64 {
                self.#ident_last_updated
            }

            fn field_names(&self) -> ::std::vec::Vec<::std::string::String> {
                vec![#(
                    ::std::string::String::from(stringify!(#field_names))
                ),*]
            }

            fn field(&self, name: &str) -> ::std::option::Option<#root::Value> {
                match name {
                    #(
                        stringify!(#field_names) => ::std::option::Option::Some(
                            #root::Value::from(self.#field_names.clone())
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_field(&mut self, name: &str, value: #root::Value) -> ::std::result::Result<#root::Value, #root::EntMutationError> {
                use ::std::convert::TryFrom;

                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as u64;

                match name {
                    #(
                        stringify!(#field_names) => {
                            let old_value = self.#field_names.clone();
                            self.#field_names = <#field_types>::try_from(value).map_err(
                                |x| #root::EntMutationError::WrongValueType { description: x.to_string() }
                            )?;
                            ::std::result::Result::Ok(#root::Value::from(old_value))
                        },
                    )*
                    _ => ::std::result::Result::Err(#root::EntMutationError::NoField {
                        name: name.to_string(),
                    }),
                }
            }

            fn edge_names(&self) -> ::std::vec::Vec<::std::string::String> {
                vec![#(
                    ::std::string::String::from(stringify!(#edge_names))
                ),*]
            }

            fn edge(&self, name: &str) -> ::std::option::Option<#root::EdgeValue> {
                match name {
                    #(
                        stringify!(#edge_names) => ::std::option::Option::Some(
                            #root::EdgeValue::from(self.#edge_names.clone())
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_edge(&mut self, name: &str, value: #root::EdgeValue) -> ::std::result::Result<#root::EdgeValue, #root::EntMutationError> {
                use ::std::convert::TryFrom;

                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as u64;

                match name {
                    #(
                        stringify!(#edge_names) => {
                            let old_value = self.#edge_names.clone();
                            self.#edge_names = <#edge_types>::try_from(value).map_err(
                                |x| #root::EntMutationError::WrongEdgeValueType { description: x.to_string() }
                            )?;
                            ::std::result::Result::Ok(#root::EdgeValue::from(old_value))
                        },
                    )*
                    _ => ::std::result::Result::Err(#root::EntMutationError::NoEdge {
                        name: name.to_string(),
                    }),
                }
            }

            fn connect(&mut self, database: ::std::boxed::Box<dyn #root::Database>) {
                self.#ident_database = ::std::option::Option::Some(database);
            }

            fn disconnect(&mut self) {
                self.#ident_database = ::std::option::Option::None;
            }

            fn is_connected(&self) -> bool {
                self.#ident_database.is_some()
            }

            fn load_edge(&self, name: &str) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::IEnt>>> {
                use #root::{Database, IEnt};
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                match self.edge(name) {
                    ::std::option::Option::Some(e) => e
                        .to_ids()
                        .into_iter()
                        .filter_map(|id| database.get(id).transpose())
                        .collect(),
                    ::std::option::Option::None => ::std::result::Result::Err(#root::DatabaseError::MissingEdge {
                        name: name.to_string(),
                    }),
                }
            }

            fn refresh(&mut self) -> #root::DatabaseResult<()> {
                use #root::{AsAny, Database, IEnt};
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                let id = self.#ident_id;
                match database.get(id)?.and_then(|ent| ent.as_any().downcast_ref::<#name>().map(::std::clone::Clone::clone)) {
                    ::std::option::Option::Some(x) => {
                        self.#ident_id = x.id();
                        self.#ident_created = x.created();
                        self.#ident_last_updated = x.last_updated();

                        #(
                            self.#field_names = x.#field_names;
                        )*

                        #(
                            self.#edge_names = x.#edge_names;
                        )*

                        ::std::result::Result::Ok(())
                    }
                    ::std::option::Option::None => ::std::result::Result::Err(#root::DatabaseError::MissingEnt { id }),
                }
            }

            fn commit(&mut self) -> #root::DatabaseResult<()> {
                use #root::{Database, IEnt};
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                match database.insert(::std::boxed::Box::from(self.clone())) {
                    ::std::result::Result::Ok(id) => {
                        self.set_id(id);
                        ::std::result::Result::Ok(())
                    }
                    ::std::result::Result::Err(x) => ::std::result::Result::Err(x),
                }
            }

            fn remove(self) -> #root::DatabaseResult<bool> {
                use #root::Database;
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                database.remove(self.#ident_id)
            }
        }
    }
}
