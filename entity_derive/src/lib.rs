use heck::ShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Field, Fields, Ident, Lit, Meta, NestedMeta,
    Type,
};

/// Transforms pseudo-struct syntax into an ent representation
///
/// ```
/// use entity::{Ent, Id, Database};
///
/// /// Define an entity and derive all associated ent functionality
/// ///
/// /// The entity must also implement clone as this is a requirement of
/// /// the IEnt trait
/// ///
/// /// If using serde, this struct will need to implement serialize and
/// /// deserialize itself
/// #[derive(Clone, Ent, serde::Serialize, serde::Deserialize)]
/// pub struct PageEnt {
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the ent's id
///     #[ent(id)]
///     id: Id,
///
///     /// Required and can only be specified once to indicate the struct
///     /// field that contains the database. Must be an option!
///     ///
///     /// If using serde, this field will need to be skipped as it will
///     /// not be serialized and, when deserializing, will be filled in
///     /// with the database automatically
///     #[ent(database)]
///     #[serde(skip)]
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
#[proc_macro_derive(Ent, attributes(ent))]
pub fn derive_ent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let root = quote! { ::entity };
    let expanded = impl_ent(root, input).unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}

#[inline]
fn impl_ent(root: TokenStream, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let span = input.span();
    let name = &input.ident;
    let vis = &input.vis;
    let const_type_name = format_ident!("{}_TYPE", name.to_string().to_shouty_snake_case());

    // If we have the attribute ent(typetag) on our struct, we will add a
    // new attribute of #[typetag::serde] onto our impl of IEnt
    let has_typetag_attr =
        input
            .attrs
            .iter()
            .filter_map(|a| a.parse_meta().ok())
            .any(|m| match m {
                Meta::List(x) if x.path.is_ident("ent") => x.nested.iter().any(|m| match m {
                    NestedMeta::Meta(x) => match x {
                        Meta::Path(x) => x.is_ident("typetag"),
                        _ => false,
                    },
                    _ => false,
                }),
                _ => false,
            });
    let typetag_t: TokenStream = if has_typetag_attr {
        quote! { #[::typetag::serde] }
    } else {
        quote! {}
    };

    let ent_inner_info = match input.data {
        Data::Struct(x) => match x.fields {
            Fields::Named(x) => extract_inner_info(span, x.named.into_iter())?,
            _ => return Err(syn::Error::new(span, "Expected named fields")),
        },
        _ => return Err(syn::Error::new(span, "Expected struct")),
    };

    let ident_id = ent_inner_info.id;
    let ident_database = ent_inner_info.database;
    let ident_created = ent_inner_info.created;
    let ident_last_updated = ent_inner_info.last_updated;
    let fields = ent_inner_info.fields;
    let edges = ent_inner_info.edges;

    let field_names: Vec<Ident> = fields.iter().map(|f| f.name.clone()).collect();
    let edge_names: Vec<Ident> = edges.iter().map(|e| e.name.clone()).collect();

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

            pub fn load_subheader(&self) -> #root::DatabaseResult<::std::option::Option<ContentEnt>> {
                todo!()
            }

            pub fn load_paragraphs(&self) -> #root::DatabaseResult<::std::vec::Vec<ContentEnt>> {
                todo!()
            }
        }

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
                use ::std::convert::TryInto;

                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as u64;

                match name {
                    #(
                        stringify!(#field_names) => {
                            let old_value = &self.#field_names;
                            self.#field_names = value.try_into().map_err(
                                |x| #root::EntMutationError::WrongValueType { description: x.to_string() }
                            )?;
                            ::std::result::Result::Ok(
                                #root::Value::from(old_value.clone())
                            )
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
                self.#ident_last_updated = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .map_err(|e| #root::EntMutationError::MarkUpdatedFailed { source: e })?
                    .as_millis() as u64;

                match name {
                    #(
                        stringify!(#edge_names) => self.#edge_names = value.try_into().map_err(
                            |x| #root::EntMutationError::WrongEdgeValueType { description: x.to_string() }
                        )?,
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
    })
}

struct EntInnerInfo {
    id: Ident,
    database: Ident,
    created: Ident,
    last_updated: Ident,
    fields: Vec<EntField>,
    edges: Vec<EntEdge>,
}

struct EntField {
    name: Ident,
    ty: Type,
    indexed: bool,
}

struct EntEdge {
    name: Ident,
    ty: Type,
    deletion_policy: EntEdgeDeletionPolicy,
}

enum EntEdgeDeletionPolicy {
    Nothing,
    Shallow,
    Deep,
}

fn extract_inner_info(
    span: Span,
    it: impl Iterator<Item = Field>,
) -> Result<EntInnerInfo, syn::Error> {
    let mut id = None;
    let mut database = None;
    let mut created = None;
    let mut last_updated = None;
    let mut fields = Vec::new();
    let mut edges = Vec::new();

    for f in it {
        let span = f.span();
        let name = f
            .ident
            .ok_or_else(|| syn::Error::new(span, "Expected named field"))?;
        let ty = f.ty;

        // Find the attribute that is ent(...), which is required on each
        // field within a struct when deriving ent
        let ent_attr_meta = f
            .attrs
            .into_iter()
            .find_map(|attr| {
                if attr.path.is_ident("ent") {
                    Some(attr.parse_meta())
                } else {
                    None
                }
            })
            .transpose()?
            .ok_or_else(|| syn::Error::new(span, "Missing ent(...) attribute"))?;

        // Grab the inner contents of ent(...) as additional meta
        let mut inner_meta_items = match ent_attr_meta {
            Meta::List(x) => x
                .nested
                .into_iter()
                .map(|x| match x {
                    NestedMeta::Meta(x) => Ok(x),
                    NestedMeta::Lit(x) => {
                        Err(syn::Error::new(x.span(), "Unexpected literal attribute"))
                    }
                })
                .collect::<Result<Vec<Meta>, syn::Error>>()?,
            _ => return Err(syn::Error::new(span, "Expected ent(...) attribute")),
        };

        if inner_meta_items.is_empty() {
            return Err(syn::Error::new(span, "Not enough items within ent(...)"));
        }

        if inner_meta_items.len() > 1 {
            return Err(syn::Error::new(span, "Too many items within ent(...)"));
        }

        match inner_meta_items.pop().unwrap() {
            // ent(id)
            Meta::Path(x) if x.is_ident("id") => {
                if id.is_some() {
                    return Err(syn::Error::new(x.span(), "Already have an id elsewhere"));
                } else {
                    id = Some(name);
                }
            }

            // ent(database)
            Meta::Path(x) if x.is_ident("database") => {
                if database.is_some() {
                    return Err(syn::Error::new(
                        x.span(),
                        "Already have a database elsewhere",
                    ));
                } else {
                    database = Some(name);
                }
            }

            // ent(created)
            Meta::Path(x) if x.is_ident("created") => {
                if created.is_some() {
                    return Err(syn::Error::new(
                        x.span(),
                        "Already have a created timestamp elsewhere",
                    ));
                } else {
                    created = Some(name);
                }
            }

            // ent(last_updated)
            Meta::Path(x) if x.is_ident("last_updated") => {
                if last_updated.is_some() {
                    return Err(syn::Error::new(
                        x.span(),
                        "Already have a last_updated timestamp elsewhere",
                    ));
                } else {
                    last_updated = Some(name);
                }
            }

            // ent(field)
            Meta::Path(x) if x.is_ident("field") => {
                fields.push(EntField {
                    name,
                    ty,
                    indexed: false,
                });
            }

            // ent(field(indexed))
            Meta::List(x) if x.path.is_ident("field") => {
                let mut indexed = false;

                for m in x.nested {
                    match m {
                        NestedMeta::Meta(x) => match x {
                            Meta::Path(x) if x.is_ident("indexed") => indexed = true,
                            x => {
                                return Err(syn::Error::new(x.span(), "Unexpected field attribute"))
                            }
                        },
                        NestedMeta::Lit(x) => {
                            return Err(syn::Error::new(x.span(), "Unexpected literal attribute"))
                        }
                    }
                }

                fields.push(EntField { name, ty, indexed });
            }

            // ent(edge)
            Meta::Path(x) if x.is_ident("edge") => {
                return Err(syn::Error::new(x.span(), "Edge attribute is missing type"))
            }

            // ent([shallow|deep], type = "...")
            Meta::List(x) if x.path.is_ident("edge") => {
                let span = x.span();
                let mut deletion_policy = EntEdgeDeletionPolicy::Nothing;
                let mut edge_type = None;

                for m in x.nested {
                    match m {
                        NestedMeta::Meta(x) => match x {
                            Meta::Path(x) if x.is_ident("nothing") => {
                                deletion_policy = EntEdgeDeletionPolicy::Nothing
                            }
                            Meta::Path(x) if x.is_ident("shallow") => {
                                deletion_policy = EntEdgeDeletionPolicy::Shallow
                            }
                            Meta::Path(x) if x.is_ident("deep") => {
                                deletion_policy = EntEdgeDeletionPolicy::Deep
                            }
                            Meta::Path(x) if x.is_ident("type") => {
                                return Err(syn::Error::new(
                                    x.span(),
                                    "Edge type must have type specified",
                                ))
                            }
                            Meta::NameValue(x) if x.path.is_ident("type") => match x.lit {
                                Lit::Str(x) => {
                                    let type_str = x.value();
                                    edge_type = Some(parse_quote!(#type_str));
                                }
                                x => {
                                    return Err(syn::Error::new(
                                        x.span(),
                                        "Unexpected edge type assignment",
                                    ))
                                }
                            },
                            x => {
                                return Err(syn::Error::new(x.span(), "Unexpected edge attribute"))
                            }
                        },
                        NestedMeta::Lit(x) => {
                            return Err(syn::Error::new(x.span(), "Unexpected literal attribute"))
                        }
                    }
                }

                edges.push(EntEdge {
                    name,
                    ty: edge_type.ok_or_else(|| syn::Error::new(span, "Missing edge type"))?,
                    deletion_policy,
                })
            }

            // For anything else, we fail because it is unsupported within ent(...)
            x => return Err(syn::Error::new(x.span(), "Unexpected ent attribute")),
        }
    }

    Ok(EntInnerInfo {
        id: id.ok_or_else(|| syn::Error::new(span, "No id field provided"))?,
        database: database.ok_or_else(|| syn::Error::new(span, "No database field provided"))?,
        created: created.ok_or_else(|| syn::Error::new(span, "No created field provided"))?,
        last_updated: last_updated
            .ok_or_else(|| syn::Error::new(span, "No last_updated field provided"))?,
        fields,
        edges,
    })
}
