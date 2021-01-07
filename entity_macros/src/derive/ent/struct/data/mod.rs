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
        let mut database = None;
        let mut created = None;
        let mut last_updated = None;
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
                }
            }

            if let Some(attr) = f.field_attr.clone().map(|a| a.unwrap_or_default()) {
                acted_on_field = true;

                fields.push(EntField {
                    name: name.clone(),
                    ty: f.ty.clone(),
                    indexed: attr.indexed,
                    mutable: attr.mutable,
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
                    wrap: attr.wrap,
                    kind,
                    deletion_policy: attr.deletion_policy,
                });
            }

            if acted_on_field {
                continue;
            } else if ent.strict {
                errors.push(darling::Error::custom("Missing ent(...) attribute").with_span(&name));
            } else {
                fields.push(EntField {
                    name: name.clone(),
                    ty: f.ty.clone(),
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
            id: id.cloned().unwrap(),
            database: database.cloned().unwrap(),
            created: created.cloned().unwrap(),
            last_updated: last_updated.cloned().unwrap(),
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
