mod internal;

use darling::{FromDeriveInput, FromMeta};
use syn::{DeriveInput, Generics, Ident, Type, Visibility};

/// Information about attributes on a struct that will represent an ent
#[derive(Debug)]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,

    pub id: Ident,
    pub id_ty: Type,

    pub database: Ident,
    pub database_ty: Type,

    pub created: Ident,
    pub created_ty: Type,

    pub last_updated: Ident,
    pub last_updated_ty: Type,

    pub fields: Vec<EntField>,
    pub edges: Vec<EntEdge>,
}

/// Extension information
#[derive(Debug, Default)]
pub struct EntExtAttr {
    pub async_graphql_filter_untyped: bool,
    pub async_graphql_filter_type: Option<Type>,
}

/// Information about a specific field for an ent
#[derive(Debug)]
pub struct EntField {
    pub name: Ident,
    pub ty: Type,

    /// If field(indexed) provided, signifies that this field should be
    /// indexed by the database where it is stored
    pub indexed: bool,

    /// If field(mutable) provided, signifies that this field should be
    /// able to be mutated and that a typed method for mutation should
    /// be included when generating typed methods
    pub mutable: bool,

    pub ext: EntExtAttr,
}

/// Information about a specific edge for an ent
#[derive(Debug)]
pub struct EntEdge {
    pub name: Ident,
    pub ty: Type,
    pub ent_ty: Type,
    pub ent_query_ty: Option<Type>,
    pub wrap: bool,
    pub kind: EntEdgeKind,
    pub deletion_policy: EntEdgeDeletionPolicy,
    pub ext: EntExtAttr,
}

/// Information about an an edge's deletion policy
#[derive(Debug, Clone, FromMeta)]
pub enum EntEdgeDeletionPolicy {
    Nothing,
    Shallow,
    Deep,
}

impl Default for EntEdgeDeletionPolicy {
    fn default() -> Self {
        Self::Nothing
    }
}

/// Information about an an edge's form
#[derive(Debug)]
pub enum EntEdgeKind {
    Maybe,
    One,
    Many,
}

impl FromDeriveInput for Ent {
    fn from_derive_input(input: &DeriveInput) -> darling::Result<Self> {
        let ent = internal::Ent::from_derive_input(input)?;
        let data_struct = ent
            .data
            .take_struct()
            .expect("Ent only supports named structs");

        let mut id = None;
        let mut id_ty = None;
        let mut database = None;
        let mut database_ty = None;
        let mut created = None;
        let mut created_ty = None;
        let mut last_updated = None;
        let mut last_updated_ty = None;
        let mut fields = Vec::new();
        let mut edges = Vec::new();

        let mut errors = vec![];

        for f in &data_struct.fields {
            // A field without the ent attribute (or with an empty attribute) will be an error
            // in strict mode or will be interpreted as a field in standard mode. In either case,
            // it's necessary to know whether any meta-items were acted on to make that fall-through
            // decision.
            let mut acted_on_field = false;

            // darling should have already validated this for us
            let name = f.ident.as_ref().expect("Ent only supports named structs");
            let ty = &f.ty;

            // A field cannot be more than one ent "thing"; it doesn't make sense for any field to be
            // both the created and last_updated values, even though they're the same data type. In the
            // interest of giving the caller a complete error list, we note this mistake but then proceed
            // to analyze the attribute in full anyway.
            if let Err(e) = f.validate_zero_or_one_known_fields() {
                errors.push(e);
            }

            if f.is_id_field() {
                acted_on_field = true;

                if id.is_some() {
                    errors.push(
                        darling::Error::custom("Already have an id elsewhere").with_span(&name),
                    );
                } else {
                    id = Some(name);
                    id_ty = Some(ty);
                }
            }

            if f.is_database_field() {
                acted_on_field = true;

                if database.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a database elsewhere")
                            .with_span(&name),
                    );
                } else {
                    database = Some(name);
                    database_ty = Some(ty);
                }
            }

            if f.is_created_field() {
                acted_on_field = true;

                if created.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a created timestamp elsewhere")
                            .with_span(&name),
                    );
                } else {
                    created = Some(name);
                    created_ty = Some(ty);
                }
            }

            if f.is_last_updated_field() {
                acted_on_field = true;

                if last_updated.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a last_updated timestamp elsewhere")
                            .with_span(&name),
                    );
                } else {
                    last_updated = Some(name);
                    last_updated_ty = Some(ty);
                }
            }

            if let Some(attr) = f.field_attr.clone().map(|a| a.unwrap_or_default()) {
                acted_on_field = true;

                fields.push(EntField {
                    name: name.clone(),
                    ty: f.ty.clone(),
                    indexed: attr.indexed.is_some(),
                    mutable: attr.mutable.is_some(),
                    ext: EntExtAttr {
                        async_graphql_filter_untyped: f
                            .ext_attr
                            .as_ref()
                            .map(|ext| ext.async_graphql.filter_untyped.is_some())
                            .unwrap_or_default(),
                        async_graphql_filter_type: f
                            .ext_attr
                            .as_ref()
                            .and_then(|ext| ext.async_graphql.filter_type.as_deref())
                            .and_then(|type_str| syn::parse_str(type_str).ok()),
                    },
                });
            }

            if let Some(attr) = f.edge_attr.clone() {
                acted_on_field = true;

                let kind = match infer_edge_kind_from_ty(&f.ty) {
                    Ok(k) => k,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };

                edges.push(EntEdge {
                    name: name.clone(),
                    ty: f.ty.clone(),
                    ent_ty: syn::parse_str(&attr.r#type)?,
                    ent_query_ty: attr
                        .query_ty
                        .and_then(|type_str| syn::parse_str(&type_str).ok()),
                    wrap: attr.wrap,
                    kind,
                    deletion_policy: attr.deletion_policy,
                    ext: EntExtAttr {
                        async_graphql_filter_untyped: f
                            .ext_attr
                            .as_ref()
                            .map(|ext| ext.async_graphql.filter_untyped.is_some())
                            .unwrap_or_default(),
                        async_graphql_filter_type: f
                            .ext_attr
                            .as_ref()
                            .and_then(|ext| ext.async_graphql.filter_type.as_deref())
                            .and_then(|type_str| syn::parse_str(type_str).ok()),
                    },
                });
            }

            if acted_on_field {
                continue;
            } else {
                fields.push(EntField {
                    name: name.clone(),
                    ty: f.ty.clone(),
                    indexed: false,
                    mutable: false,
                    ext: Default::default(),
                });
            }
        }

        if id.is_none() {
            errors.push(darling::Error::custom("No id field provided").with_span(input));
        }

        if database.is_none() {
            errors.push(darling::Error::custom("No database field provided").with_span(input));
        }

        if created.is_none() {
            errors.push(darling::Error::custom("No created field provided").with_span(input));
        }

        if last_updated.is_none() {
            errors.push(darling::Error::custom("No last_updated field provided").with_span(input));
        }

        if !errors.is_empty() {
            return Err(darling::Error::multiple(errors));
        }

        Ok(Ent {
            ident: ent.ident,
            vis: ent.vis,
            generics: ent.generics,
            // These unwraps are safe because the previous is_none() checks should
            // have caused a return before reaching this point.
            id: id.cloned().unwrap(),
            id_ty: id_ty.cloned().unwrap(),
            database: database.cloned().unwrap(),
            database_ty: database_ty.cloned().unwrap(),
            created: created.cloned().unwrap(),
            created_ty: created_ty.cloned().unwrap(),
            last_updated: last_updated.cloned().unwrap(),
            last_updated_ty: last_updated_ty.cloned().unwrap(),
            fields,
            edges,
        })
    }
}

fn infer_edge_kind_from_ty(ty: &Type) -> darling::Result<EntEdgeKind> {
    match &ty {
        Type::Path(x) => {
            let segment = match x.path.segments.last() {
                Some(seg) => seg,
                None => {
                    return Err(darling::Error::custom("Missing edge id type").with_span(x));
                }
            };
            Ok(match segment.ident.to_string().to_lowercase().as_str() {
                "option" => EntEdgeKind::Maybe,
                "vec" => EntEdgeKind::Many,
                _ => EntEdgeKind::One,
            })
        }
        x => Err(darling::Error::custom("Unexpected edge id type").with_span(x)),
    }
}
