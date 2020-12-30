mod internal;

use darling::{FromDeriveInput, FromMeta};
use std::convert::TryFrom;
use syn::{spanned::Spanned, DeriveInput, Generics, Ident, Type, Visibility};

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
    pub ent_wrap_types: Vec<Type>,
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

impl TryFrom<&DeriveInput> for Ent {
    type Error = syn::Error;

    fn try_from(input: &DeriveInput) -> Result<Self, Self::Error> {
        let ent = internal::Ent::from_derive_input(input)
            .map_err(|e| syn::Error::new(input.span(), e.to_string()))?;

        let mut id = None;
        let mut database = None;
        let mut created = None;
        let mut last_updated = None;
        let mut fields = Vec::new();
        let mut edges = Vec::new();

        for f in ent.data.take_struct().unwrap().fields {
            let name = f.ident.unwrap();
            let ty = f.ty;

            if f.is_ent_id_field {
                if id.is_some() {
                    return Err(syn::Error::new(name.span(), "Already have an id elsewhere"));
                } else {
                    id = Some(name);
                }
            } else if f.is_ent_database_field {
                if database.is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "Already have a database elsewhere",
                    ));
                } else {
                    database = Some(name);
                }
            } else if f.is_ent_created_field {
                if created.is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "Already have a created timestamp elsewhere",
                    ));
                } else {
                    created = Some(name);
                }
            } else if f.is_ent_last_updated_field {
                if last_updated.is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "Already have a last_updated timestamp elsewhere",
                    ));
                } else {
                    last_updated = Some(name);
                }
            } else if let Some(attr) = f.field_attr {
                fields.push(EntField {
                    name,
                    ty,
                    indexed: attr.indexed,
                    mutable: attr.mutable,
                });
            } else if let Some(attr) = f.edge_attr {
                let wrapping_types = attr
                    .wrapping_types
                    .into_iter()
                    .map(|t| syn::parse_str(&t))
                    .collect::<Result<Vec<Type>, syn::Error>>()?;
                let kind = match &ty {
                    Type::Path(x) => {
                        let segment = x
                            .path
                            .segments
                            .last()
                            .ok_or_else(|| syn::Error::new(x.span(), "Missing edge id type"))?;
                        match segment.ident.to_string().to_lowercase().as_str() {
                            "option" => EntEdgeKind::Maybe,
                            "vec" => EntEdgeKind::Many,
                            _ => EntEdgeKind::One,
                        }
                    }
                    x => return Err(syn::Error::new(x.span(), "Unexpected edge id type")),
                };

                edges.push(EntEdge {
                    name,
                    ty,
                    ent_ty: syn::parse_str(&attr.r#type)?,
                    ent_wrap_types: wrapping_types,
                    kind,
                    deletion_policy: attr.deletion_policy,
                });
            } else if ent.strict {
                return Err(syn::Error::new(name.span(), "Missing ent(...) attribute"));
            } else {
                fields.push(EntField {
                    name,
                    ty,
                    indexed: false,
                    mutable: false,
                });
            }
        }

        Ok(Ent {
            ident: ent.ident,
            vis: ent.vis,
            generics: ent.generics,
            id: id.ok_or_else(|| syn::Error::new(input.span(), "No id field provided"))?,
            database: database
                .ok_or_else(|| syn::Error::new(input.span(), "No database field provided"))?,
            created: created
                .ok_or_else(|| syn::Error::new(input.span(), "No created field provided"))?,
            last_updated: last_updated
                .ok_or_else(|| syn::Error::new(input.span(), "No last_updated field provided"))?,
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
