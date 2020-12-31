use super::{Ent, EntEdge, EntEdgeDeletionPolicy, EntEdgeKind, EntField};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Generics, Ident, Path, Type};

pub(crate) fn impl_ent(
    root: &Path,
    name: &Ident,
    generics: &Generics,
    ent: &Ent,
    const_type_name: &Ident,
    include_typetag: bool,
) -> Result<TokenStream, syn::Error> {
    let ident_id = &ent.id;
    let ident_database = &ent.database;
    let ident_created = &ent.created;
    let ident_last_updated = &ent.last_updated;
    let fields = &ent.fields;
    let edges = &ent.edges;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_names: Vec<Ident> = fields.iter().map(|f| f.name.clone()).collect();
    let field_types: Vec<Type> = fields.iter().map(|f| f.ty.clone()).collect();
    let value_to_typed_field: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let value_ident = Ident::new("value", Span::call_site());
            let assign_value = utils::convert_from_value(&value_ident, &f.ty);
            quote! { #assign_value }
        })
        .collect();
    let edge_names: Vec<Ident> = edges.iter().map(|e| e.name.clone()).collect();
    let edge_types: Vec<Type> = edges.iter().map(|e| e.ty.clone()).collect();

    let field_definitions = make_field_definitions(root, fields)?;
    let edge_definitions = make_edge_definitions(root, edges);

    // If we have the attribute ent(typetag) on our struct, we will add a
    // new attribute of #[typetag::serde] onto our impl of Ent
    let typetag_t: TokenStream = if include_typetag {
        quote! { #[::typetag::serde] }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #root::EntType for #name #ty_generics #where_clause {
            fn type_str() -> &'static str {
                #const_type_name
            }
        }

        #typetag_t
        #[automatically_derived]
        impl #impl_generics #root::Ent for #name #ty_generics #where_clause {
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

            fn mark_updated(&mut self) -> Result<(), #root::EntMutationError> {
                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as u64;
                Ok(())
            }

            fn field_definitions(&self) -> ::std::vec::Vec<#root::FieldDefinition> {
                vec![#(#field_definitions),*]
            }

            fn field(&self, name: &str) -> ::std::option::Option<#root::Value> {
                match name {
                    #(
                        stringify!(#field_names) => ::std::option::Option::Some(
                            self.#field_names.clone().into()
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_field(&mut self, name: &str, value: #root::Value) -> ::std::result::Result<#root::Value, #root::EntMutationError> {
                match name {
                    #(
                        stringify!(#field_names) => {
                            let old_value = self.#field_names.clone();
                            let converted: ::std::result::Result<#field_types, &'static str> = #value_to_typed_field;
                            self.#field_names = converted.map_err(
                                |x| #root::EntMutationError::WrongValueType { description: x.to_string() }
                            )?;
                            ::std::result::Result::Ok(old_value.into())
                        },
                    )*
                    _ => ::std::result::Result::Err(#root::EntMutationError::NoField {
                        name: name.to_string(),
                    }),
                }
            }

            fn edge_definitions(&self) -> ::std::vec::Vec<#root::EdgeDefinition> {
                vec![#(#edge_definitions),*]
            }

            fn edge(&self, name: &str) -> ::std::option::Option<#root::EdgeValue> {
                match name {
                    #(
                        stringify!(#edge_names) => ::std::option::Option::Some(
                            self.#edge_names.clone().into()
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_edge(&mut self, name: &str, value: #root::EdgeValue) -> ::std::result::Result<#root::EdgeValue, #root::EntMutationError> {
                use ::std::convert::TryFrom;

                match name {
                    #(
                        stringify!(#edge_names) => {
                            let old_value = self.#edge_names.clone();
                            self.#edge_names = <#edge_types>::try_from(value).map_err(
                                |x| #root::EntMutationError::WrongEdgeValueType { description: x.to_string() }
                            )?;
                            ::std::result::Result::Ok(old_value.into())
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

            fn load_edge(&self, name: &str) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::Ent>>> {
                use #root::{Database, Ent};
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
                use #root::{AsAny, Database, Ent};
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                let id = self.#ident_id;
                match database.get(id)?.and_then(
                    |ent| ent.as_any()
                        .downcast_ref::<#name #ty_generics>()
                        .map(::std::clone::Clone::clone)
                ) {
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
                use #root::{Database, Ent};
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                match database.insert(::std::boxed::Box::from(self.clone())) {
                    ::std::result::Result::Ok(id) => {
                        self.set_id(id);
                        ::std::result::Result::Ok(())
                    }
                    ::std::result::Result::Err(x) => ::std::result::Result::Err(x),
                }
            }

            fn remove(&self) -> #root::DatabaseResult<bool> {
                use #root::Database;
                let database = self.#ident_database.as_ref().ok_or(#root::DatabaseError::Disconnected)?;
                database.remove(self.#ident_id)
            }
        }
    })
}

fn make_field_definitions(
    root: &Path,
    fields: &[EntField],
) -> Result<Vec<TokenStream>, syn::Error> {
    let mut token_streams = Vec::new();

    for f in fields {
        let name = &f.name;
        let value_type = make_field_value_type(root, &f.ty)?;

        let mut attrs = Vec::new();

        if f.indexed {
            attrs.push(quote! { #root::FieldAttribute::Indexed });
        }

        if !f.mutable {
            attrs.push(quote! { #root::FieldAttribute::Immutable });
        }

        token_streams.push(quote! {
            #root::FieldDefinition::new_with_attributes(
                stringify!(#name),
                #value_type,
                vec![#(#attrs),*],
            )
        });
    }

    Ok(token_streams)
}

fn make_field_value_type(root: &Path, r#type: &Type) -> Result<TokenStream, syn::Error> {
    Ok(match r#type {
        Type::Path(x) => {
            if let Some(seg) = x.path.segments.last() {
                match seg.ident.to_string().to_lowercase().as_str() {
                    "vec" => {
                        let inner = utils::get_inner_type_from_segment(seg, 0, 1)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::List(::std::boxed::Box::from(#inner_t))
                        }
                    }
                    "hashmap" => {
                        let inner = utils::get_inner_type_from_segment(seg, 1, 2)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::Map(::std::boxed::Box::from(#inner_t))
                        }
                    }
                    "option" => {
                        let inner = utils::get_inner_type_from_segment(seg, 0, 1)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::Optional(::std::boxed::Box::from(#inner_t))
                        }
                    }
                    "string" => quote! { #root::ValueType::Text },
                    "()" => quote! { #root::PrimitiveValueType::Unit },
                    "bool" => quote! { #root::PrimitiveValueType::Bool },
                    "char" => quote! { #root::PrimitiveValueType::Char },
                    "f32" => quote! { #root::NumberType::F32 },
                    "f64" => quote! { #root::NumberType::F64 },
                    "i128" => quote! { #root::NumberType::I128 },
                    "i16" => quote! { #root::NumberType::I16 },
                    "i32" => quote! { #root::NumberType::I32 },
                    "i64" => quote! { #root::NumberType::I64 },
                    "i8" => quote! { #root::NumberType::I8 },
                    "isize" => quote! { #root::NumberType::Isize },
                    "u128" => quote! { #root::NumberType::U128 },
                    "u16" => quote! { #root::NumberType::U16 },
                    "u32" => quote! { #root::NumberType::U32 },
                    "u64" => quote! { #root::NumberType::U64 },
                    "u8" => quote! { #root::NumberType::U8 },
                    "usize" => quote! { #root::NumberType::Usize },
                    _ => quote! { #root::ValueType::Custom },
                }
            } else {
                return Err(syn::Error::new(
                    r#type.span(),
                    "Missing last segment in type path",
                ));
            }
        }
        Type::Array(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "Arrays are not supported as field types",
            ))
        }
        Type::BareFn(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "fn(...) are not supported as field types",
            ))
        }
        Type::Ptr(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "*const T and *mut T are not supported as field types",
            ))
        }
        Type::Reference(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "&'a T and &'a mut T are not supported as field types",
            ))
        }
        Type::Slice(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "Slices are not supported as field types",
            ))
        }
        Type::Tuple(_) => {
            return Err(syn::Error::new(
                r#type.span(),
                "Tuples are not supported as field types",
            ))
        }
        _ => return Err(syn::Error::new(r#type.span(), "Unexpected type format")),
    })
}

fn make_edge_definitions(root: &Path, edges: &[EntEdge]) -> Vec<TokenStream> {
    let mut token_streams = Vec::new();

    for e in edges {
        let name = &e.name;
        let ty = match e.kind {
            EntEdgeKind::Many => quote! { #root::EdgeValueType::Many },
            EntEdgeKind::Maybe => quote! { #root::EdgeValueType::MaybeOne },
            EntEdgeKind::One => quote! { #root::EdgeValueType::One },
        };
        let deletion_policy = match e.deletion_policy {
            EntEdgeDeletionPolicy::Deep => quote! { #root::EdgeDeletionPolicy::DeepDelete },
            EntEdgeDeletionPolicy::Shallow => quote! { #root::EdgeDeletionPolicy::ShallowDelete },
            EntEdgeDeletionPolicy::Nothing => quote! { #root::EdgeDeletionPolicy::Nothing },
        };

        token_streams.push(quote! {
            #root::EdgeDefinition::new_with_deletion_policy(
                stringify!(#name),
                #ty,
                #deletion_policy,
            )
        });
    }

    token_streams
}
