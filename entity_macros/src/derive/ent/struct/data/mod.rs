mod internal;

use darling::{FromDeriveInput, FromMeta};
use syn::{DeriveInput, Generics, Ident, Type, Visibility};

/// Information about attributes on a struct that will represent an ent
#[derive(Debug)]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub attr: EntAttr,
    pub id: Ident,
    pub database: Ident,
    pub created: Ident,
    pub last_updated: Ident,
    pub fields: Vec<EntField>,
    pub edges: Vec<EntEdge>,
}

/// Struct type-level attributes for an ent
#[derive(Debug)]
pub struct EntAttr {
    /// Indicates not to generate a builder helper struct
    pub no_builder: bool,

    /// Indicates not to generate a typed query struct
    pub no_query: bool,

    /// Indicates not to generate typed methods to access and
    /// mutate ent fields
    pub no_typed_methods: bool,

    /// Indicates to include the typetag attribute on the ent trait impl,
    /// required only when Serialize/Deserialize from serde is being
    /// implemented for the given type
    pub typetag: bool,

    /// Indicates that struct fields must be explicitly labeled
    /// as a an ent field or edge, rather than defaulting to ent field when
    /// unlabeled
    pub strict: bool,
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
}

/// Information about a specific edge for an ent
#[derive(Debug)]
pub struct EntEdge {
    pub name: Ident,
    pub ty: Type,
    pub ent_ty: Type,
    pub wrap: bool,
    pub kind: EntEdgeKind,
    pub deletion_policy: EntEdgeDeletionPolicy,
}

/// Information about an an edge's deletion policy
#[derive(Debug, FromMeta)]
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

        let mut id = None;
        let mut database = None;
        let mut created = None;
        let mut last_updated = None;
        let mut fields = Vec::new();
        let mut edges = Vec::new();

        let mut errors = vec![];

        for f in ent.data.take_struct().unwrap().fields {
            let name = f.ident.unwrap();
            let ty = f.ty;

            if f.is_ent_id_field {
                if id.is_some() {
                    errors.push(
                        darling::Error::custom("Already have an id elsewhere").with_span(&name),
                    );
                } else {
                    id = Some(name);
                }
            } else if f.is_ent_database_field {
                if database.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a database elsewhere")
                            .with_span(&name),
                    );
                } else {
                    database = Some(name);
                }
            } else if f.is_ent_created_field {
                if created.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a created timestamp elsewhere")
                            .with_span(&name),
                    );
                } else {
                    created = Some(name);
                }
            } else if f.is_ent_last_updated_field {
                if last_updated.is_some() {
                    errors.push(
                        darling::Error::custom("Already have a last_updated timestamp elsewhere")
                            .with_span(&name),
                    );
                } else {
                    last_updated = Some(name);
                }
            } else if let Some(attr) = f.field_attr.map(|a| a.unwrap_or_default()) {
                fields.push(EntField {
                    name,
                    ty,
                    indexed: attr.indexed,
                    mutable: attr.mutable,
                });
            } else if let Some(attr) = f.edge_attr {
                let kind = match infer_edge_kind_from_ty(&ty) {
                    Ok(k) => k,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };

                edges.push(EntEdge {
                    name,
                    ty,
                    ent_ty: syn::parse_str(&attr.r#type)?,
                    wrap: attr.wrap,
                    kind,
                    deletion_policy: attr.deletion_policy,
                });
            } else if ent.strict {
                errors.push(darling::Error::custom("Missing ent(...) attribute").with_span(&name));
            } else {
                fields.push(EntField {
                    name,
                    ty,
                    indexed: false,
                    mutable: false,
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
            id: id.unwrap(),
            database: database.unwrap(),
            created: created.unwrap(),
            last_updated: last_updated.unwrap(),
            fields,
            edges,
            attr: EntAttr {
                no_builder: ent.no_builder,
                no_query: ent.no_query,
                no_typed_methods: ent.no_typed_methods,
                typetag: ent.typetag,
                strict: ent.strict,
            },
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
