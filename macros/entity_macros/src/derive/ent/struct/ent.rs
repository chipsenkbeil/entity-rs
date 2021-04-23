use crate::utils;
use entity_macros_data::{
    StructEnt, StructEntEdge, StructEntEdgeDeletionPolicy, StructEntEdgeKind, StructEntField,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Expr, Ident, Path, Type};

pub fn do_derive_ent(root: Path, ent: StructEnt) -> darling::Result<TokenStream> {
    let name = &ent.ident;

    let ident_id = &ent.id;
    let ident_database = &ent.database;
    let ident_created = &ent.created;
    let ident_last_updated = &ent.last_updated;
    let fields = &ent.fields;
    let edges = &ent.edges;

    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    let field_names: Vec<Ident> = fields.iter().map(|f| f.name.clone()).collect();
    let field_values: Vec<Expr> = fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            if let Some(computed) = f.computed.as_ref() {
                computed.expr.clone()
            } else {
                parse_quote!(::std::clone::Clone::clone(&self.#field_name))
            }
        })
        .collect();
    let value_to_typed_field: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let value_ident = Ident::new("value", Span::call_site());
            let assign_value = utils::convert_from_value(&root, &value_ident, &f.ty);
            quote! { #assign_value }
        })
        .collect();
    let update_fields: Vec<TokenStream> = fields
        .iter()
        .zip(value_to_typed_field.iter())
        .map(|(f, v_to_tf)| {
            let field_name = &f.name;
            let field_ty = &f.ty;

            let body = if f.computed.is_some() {
                quote!(::std::result::Result::Err(
                    #root::EntMutationError::FieldComputed {
                        name: ::std::string::ToString::to_string(
                            ::std::stringify!(#field_name)
                        )
                    }
                ))
            } else if !f.mutable {
                quote!(::std::result::Result::Err(
                    #root::EntMutationError::FieldImmutable {
                        name: ::std::string::ToString::to_string(
                            ::std::stringify!(#field_name)
                        )
                    }
                ))
            } else {
                quote!({
                    let old_value = ::std::clone::Clone::clone(&self.#field_name);
                    let converted: ::std::result::Result<
                        #field_ty,
                        #root::Value,
                    > = #v_to_tf;
                    self.#field_name = converted.map_err(
                        |_| #root::EntMutationError::WrongValueType {
                            description: ::std::string::ToString::to_string(
                                ::std::concat!(
                                    "Value is not ",
                                    ::std::stringify!(#field_ty),
                                )
                            )
                        }
                    )?;
                    ::std::result::Result::Ok(#root::ValueLike::into_value(old_value))
                })
            };

            quote!(::std::stringify!(#field_name) => {#body})
        })
        .collect();
    let clear_cache_expr: TokenStream = {
        let clearings: Vec<TokenStream> = fields
            .iter()
            .filter_map(|f| {
                if f.computed.is_some() {
                    let field_name = &f.name;
                    Some(quote!(self.#field_name = ::std::option::Option::None;))
                } else {
                    None
                }
            })
            .collect();

        quote!({
            #(#clearings)*
        })
    };

    let edge_names: Vec<Ident> = edges.iter().map(|e| e.name.clone()).collect();
    let edge_types: Vec<Type> = edges.iter().map(|e| e.ty.clone()).collect();

    let field_definitions = make_field_definitions(&root, fields)?;
    let edge_definitions = make_edge_definitions(&root, edges);

    let typetag_root = utils::typetag_crate()?;
    let typetag_t = quote!(#[#typetag_root::serde]);

    let type_str_t = utils::make_type_str(name);

    Ok(quote! {
        #typetag_t
        #[automatically_derived]
        impl #impl_generics #root::Ent for #name #ty_generics #where_clause {
            fn id(&self) -> #root::Id {
                self.#ident_id
            }

            fn set_id(&mut self, id: #root::Id) {
                self.#ident_id = id;
            }

            fn r#type(&self) -> &::std::primitive::str {
                #type_str_t
            }

            fn created(&self) -> ::std::primitive::u64 {
                self.#ident_created
            }

            fn last_updated(&self) -> ::std::primitive::u64 {
                self.#ident_last_updated
            }

            fn mark_updated(&mut self) -> ::std::result::Result<(), #root::EntMutationError> {
                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as ::std::primitive::u64;
                ::std::result::Result::Ok(())
            }

            fn field_definitions(&self) -> ::std::vec::Vec<#root::FieldDefinition> {
                let mut x = ::std::vec::Vec::new();
                #(
                    x.push(#field_definitions);
                )*
                x
            }

            fn field(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::Value> {
                match name {
                    #(
                        ::std::stringify!(#field_names) => ::std::option::Option::Some(
                            #root::ValueLike::into_value(#field_values)
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_field(
                &mut self,
                name: &::std::primitive::str,
                value: #root::Value,
            ) -> ::std::result::Result<#root::Value, #root::EntMutationError> {
                match name {
                    #(#update_fields),*
                    _ => ::std::result::Result::Err(#root::EntMutationError::NoField {
                        name: ::std::string::ToString::to_string(name),
                    }),
                }
            }

            fn edge_definitions(&self) -> ::std::vec::Vec<#root::EdgeDefinition> {
                let mut x = ::std::vec::Vec::new();
                #(
                    x.push(#edge_definitions);
                )*
                x
            }

            fn edge(&self, name: &::std::primitive::str) -> ::std::option::Option<#root::EdgeValue> {
                match name {
                    #(
                        ::std::stringify!(#edge_names) => ::std::option::Option::Some(
                            ::std::convert::Into::<#root::EdgeValue>::into(
                                ::std::clone::Clone::clone(&self.#edge_names)
                            )
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            fn update_edge(
                &mut self,
                name: &::std::primitive::str,
                value: #root::EdgeValue,
            ) -> ::std::result::Result<#root::EdgeValue, #root::EntMutationError> {
                match name {
                    #(
                        ::std::stringify!(#edge_names) => {
                            let old_value = ::std::clone::Clone::clone(&self.#edge_names);
                            self.#edge_names = <
                                #edge_types as ::std::convert::TryFrom<#root::EdgeValue>
                            >::try_from(value).map_err(
                                |x| #root::EntMutationError::WrongEdgeValueType {
                                    description: ::std::string::ToString::to_string(&x)
                                }
                            )?;
                            ::std::result::Result::Ok(
                                ::std::convert::Into::<#root::EdgeValue>::into(old_value)
                            )
                        },
                    )*
                    _ => ::std::result::Result::Err(#root::EntMutationError::NoEdge {
                        name: ::std::string::ToString::to_string(name),
                    }),
                }
            }

            fn connect(&mut self, database: #root::WeakDatabaseRc) {
                self.#ident_database = database;
            }

            fn disconnect(&mut self) {
                self.#ident_database = #root::WeakDatabaseRc::new();
            }

            fn is_connected(&self) -> ::std::primitive::bool {
                #root::WeakDatabaseRc::strong_count(&self.#ident_database) > 0
            }

            fn load_edge(
                &self,
                name: &::std::primitive::str,
            ) -> #root::DatabaseResult<::std::vec::Vec<::std::boxed::Box<dyn #root::Ent>>> {
                let database = #root::WeakDatabaseRc::upgrade(
                    &self.#ident_database
                ).ok_or(#root::DatabaseError::Disconnected)?;
                match #root::Ent::edge(self, name) {
                    ::std::option::Option::Some(e) =>
                        ::std::iter::Iterator::collect(
                            ::std::iter::Iterator::filter_map(
                                ::std::iter::IntoIterator::into_iter(e.to_ids()),
                                |id| #root::Database::get(
                                    ::std::convert::AsRef::<dyn #root::Database>::as_ref(
                                        ::std::convert::AsRef::<
                                            ::std::boxed::Box<dyn #root::Database>
                                        >::as_ref(&database),
                                    ),
                                    id,
                                ).transpose(),
                            )
                        ),
                    ::std::option::Option::None => ::std::result::Result::Err(#root::DatabaseError::MissingEdge {
                        name: ::std::string::ToString::to_string(name),
                    }),
                }
            }

            fn clear_cache(&mut self) {
                #clear_cache_expr
            }

            fn refresh(&mut self) -> #root::DatabaseResult<()> {
                let database = #root::WeakDatabaseRc::upgrade(
                    &self.#ident_database
                ).ok_or(#root::DatabaseError::Disconnected)?;
                let id = self.#ident_id;

                match #root::Database::get(
                    ::std::convert::AsRef::<dyn #root::Database>::as_ref(
                        ::std::convert::AsRef::<
                            ::std::boxed::Box<dyn #root::Database>
                        >::as_ref(&database),
                    ),
                    id,
                )?.and_then(|ent| ent.to_ent::<#name #ty_generics>()) {
                    ::std::option::Option::Some(x) => {
                        self.#ident_id = #root::Ent::id(&x);
                        self.#ident_created = #root::Ent::created(&x);
                        self.#ident_last_updated = #root::Ent::last_updated(&x);

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
                let database = #root::WeakDatabaseRc::upgrade(
                    &self.#ident_database
                ).ok_or(#root::DatabaseError::Disconnected)?;
                match #root::Database::insert(
                    ::std::convert::AsRef::<dyn #root::Database>::as_ref(
                        ::std::convert::AsRef::<
                            ::std::boxed::Box<dyn #root::Database>
                        >::as_ref(&database),
                    ),
                    ::std::boxed::Box::new(
                        ::std::clone::Clone::clone(
                            ::std::ops::Deref::deref(&self)
                        )
                    ),
                ) {
                    ::std::result::Result::Ok(id) => {
                        #root::Ent::set_id(self, id);
                        ::std::result::Result::Ok(())
                    }
                    ::std::result::Result::Err(x) => ::std::result::Result::Err(x),
                }
            }

            fn remove(&self) -> #root::DatabaseResult<::std::primitive::bool> {
                let database = #root::WeakDatabaseRc::upgrade(
                    &self.#ident_database
                ).ok_or(#root::DatabaseError::Disconnected)?;
                #root::Database::remove(
                    ::std::convert::AsRef::<dyn #root::Database>::as_ref(
                        ::std::convert::AsRef::<
                            ::std::boxed::Box<dyn #root::Database>
                        >::as_ref(&database),
                    ),
                    self.#ident_id,
                )
            }
        }
    })
}

fn make_field_definitions(
    root: &Path,
    fields: &[StructEntField],
) -> darling::Result<Vec<TokenStream>> {
    let mut token_streams = Vec::new();
    let mut errors = Vec::new();

    for f in fields {
        let name = &f.name;
        let value_type = match make_field_value_type(root, &f.ty) {
            Ok(x) => x,
            Err(x) => {
                errors.push(x);
                continue;
            }
        };

        let mut attrs = Vec::new();

        if f.indexed {
            attrs.push(quote! { #root::FieldAttribute::Indexed });
        }

        if !f.mutable {
            attrs.push(quote! { #root::FieldAttribute::Immutable });
        }

        if f.computed.is_some() {
            attrs.push(quote! { #root::FieldAttribute::Computed });
        }

        token_streams.push(quote! {
            #root::FieldDefinition::new_with_attributes(
                ::std::stringify!(#name),
                #value_type,
                {
                    let mut x = ::std::vec::Vec::new();
                    #(
                        x.push(#attrs);
                    )*
                    x
                },
            )
        });
    }

    if errors.is_empty() {
        Ok(token_streams)
    } else {
        Err(darling::Error::multiple(errors))
    }
}

/// Given some syn [`Type`], will produce an entity `ValueType`
fn make_field_value_type(root: &Path, r#type: &Type) -> darling::Result<TokenStream> {
    Ok(match r#type {
        Type::Path(x) => {
            if let Some(seg) = x.path.segments.last() {
                match seg.ident.to_string().to_lowercase().as_str() {
                    "vec" | "vecdeque" | "linkedlist" | "binaryheap" | "hashset" | "btreeset" => {
                        let inner = utils::get_inner_type_from_segment(seg, 0, 1)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::List(::std::boxed::Box::new(#inner_t))
                        }
                    }
                    "hashmap" | "btreemap" => {
                        let inner = utils::get_inner_type_from_segment(seg, 1, 2)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::Map(::std::boxed::Box::new(#inner_t))
                        }
                    }
                    "option" => {
                        let inner = utils::get_inner_type_from_segment(seg, 0, 1)?;
                        let inner_t = make_field_value_type(root, inner)?;

                        quote! {
                            #root::ValueType::Optional(::std::boxed::Box::new(#inner_t))
                        }
                    }
                    "string" | "pathbuf" | "osstring" => quote! { #root::ValueType::Text },
                    "()" => quote! { #root::ValueType::Primitive(#root::PrimitiveType::Unit) },
                    "bool" => quote! { #root::ValueType::Primitive(#root::PrimitiveType::Bool) },
                    "char" => quote! { #root::ValueType::Primitive(#root::PrimitiveType::Char) },
                    "f32" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::F32)) }
                    }
                    "f64" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::F64)) }
                    }
                    "i128" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::I128)) }
                    }
                    "i16" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::I16)) }
                    }
                    "i32" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::I32)) }
                    }
                    "i64" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::I64)) }
                    }
                    "i8" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::I8)) }
                    }
                    "isize" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::Isize)) }
                    }
                    "u128" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::U128)) }
                    }
                    "u16" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::U16)) }
                    }
                    "u32" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::U32)) }
                    }
                    "u64" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::U64)) }
                    }
                    "u8" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::U8)) }
                    }
                    "usize" => {
                        quote! { #root::ValueType::Primitive(#root::PrimitiveType::Number(#root::NumberType::Usize)) }
                    }
                    _ => quote! { #root::ValueType::Custom },
                }
            } else {
                return Err(
                    darling::Error::custom("Missing last segment in type path").with_span(r#type)
                );
            }
        }
        Type::Array(_) => {
            return Err(
                darling::Error::custom("Arrays are not supported as field types").with_span(r#type),
            )
        }
        Type::BareFn(_) => {
            return Err(
                darling::Error::custom("fn(...) are not supported as field types")
                    .with_span(r#type),
            )
        }
        Type::Ptr(_) => {
            return Err(darling::Error::custom(
                "*const T and *mut T are not supported as field types",
            )
            .with_span(r#type))
        }
        Type::Reference(_) => {
            return Err(darling::Error::custom(
                "&'a T and &'a mut T are not supported as field types",
            )
            .with_span(r#type))
        }
        Type::Slice(_) => {
            return Err(
                darling::Error::custom("Slices are not supported as field types").with_span(r#type),
            )
        }
        Type::Tuple(_) => {
            return Err(
                darling::Error::custom("Tuples are not supported as field types").with_span(r#type),
            )
        }
        _ => return Err(darling::Error::custom("Unexpected type format").with_span(r#type)),
    })
}

fn make_edge_definitions(root: &Path, edges: &[StructEntEdge]) -> Vec<TokenStream> {
    let mut token_streams = Vec::new();

    for e in edges {
        let name = &e.name;
        let ty = match e.kind {
            StructEntEdgeKind::Many => quote! { #root::EdgeValueType::Many },
            StructEntEdgeKind::Maybe => quote! { #root::EdgeValueType::MaybeOne },
            StructEntEdgeKind::One => quote! { #root::EdgeValueType::One },
        };
        let deletion_policy = match e.deletion_policy {
            StructEntEdgeDeletionPolicy::Deep => quote! { #root::EdgeDeletionPolicy::DeepDelete },
            StructEntEdgeDeletionPolicy::Shallow => {
                quote! { #root::EdgeDeletionPolicy::ShallowDelete }
            }
            StructEntEdgeDeletionPolicy::Nothing => quote! { #root::EdgeDeletionPolicy::Nothing },
        };

        token_streams.push(quote! {
            #root::EdgeDefinition::new_with_deletion_policy(
                ::std::stringify!(#name),
                #ty,
                #deletion_policy,
            )
        });
    }

    token_streams
}
