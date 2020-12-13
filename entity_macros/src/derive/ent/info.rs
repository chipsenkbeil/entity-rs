use super::utils;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident, Lit, Meta, NestedMeta, Type};

/// Information about attributes on a struct that will represent an ent
#[derive(Debug)]
pub struct EntInfo {
    pub id: Ident,
    pub database: Ident,
    pub created: Ident,
    pub last_updated: Ident,
    pub fields: Vec<EntField>,
    pub edges: Vec<EntEdge>,
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
    pub kind: EntEdgeKind,
    pub deletion_policy: EntEdgeDeletionPolicy,
}

/// Information about an an edge's deletion policy
#[derive(Debug)]
pub enum EntEdgeDeletionPolicy {
    Nothing,
    Shallow,
    Deep,
}

/// Information about an an edge's form
#[derive(Debug)]
pub enum EntEdgeKind {
    Maybe,
    One,
    Many,
}

impl TryFrom<&DeriveInput> for EntInfo {
    type Error = syn::Error;

    fn try_from(input: &DeriveInput) -> Result<Self, Self::Error> {
        let span = input.span();
        let named_fields = &utils::get_named_fields(input)?.named;

        let mut id = None;
        let mut database = None;
        let mut created = None;
        let mut last_updated = None;
        let mut fields = Vec::new();
        let mut edges = Vec::new();

        for f in named_fields {
            let span = f.span();
            let name = f.ident.clone().unwrap();
            let ty = f.ty.clone();

            // Find the attribute that is ent(...), which is required on each
            // field within a struct when deriving ent
            let ent_attr_meta = f
                .attrs
                .iter()
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
                        mutable: false,
                    });
                }

                // ent(field([indexed], [mutable]))
                Meta::List(x) if x.path.is_ident("field") => {
                    let mut indexed = false;
                    let mut mutable = false;

                    for m in x.nested {
                        match m {
                            NestedMeta::Meta(x) => match x {
                                Meta::Path(x) if x.is_ident("indexed") => indexed = true,
                                Meta::Path(x) if x.is_ident("mutable") => mutable = true,
                                x => {
                                    return Err(syn::Error::new(
                                        x.span(),
                                        "Unexpected field attribute",
                                    ))
                                }
                            },
                            NestedMeta::Lit(x) => {
                                return Err(syn::Error::new(
                                    x.span(),
                                    "Unexpected literal attribute",
                                ))
                            }
                        }
                    }

                    fields.push(EntField {
                        name,
                        ty,
                        indexed,
                        mutable,
                    });
                }

                // ent(edge)
                Meta::Path(x) if x.is_ident("edge") => {
                    return Err(syn::Error::new(x.span(), "Edge attribute is missing type"));
                }

                // ent([shallow|deep], type = "...")
                Meta::List(x) if x.path.is_ident("edge") => {
                    let span = x.span();
                    let mut deletion_policy = EntEdgeDeletionPolicy::Nothing;
                    let mut edge_type = None;

                    // Figure out edge type (Maybe/One/Many) based on
                    // (Option<...>, ..., Vec<...>)
                    let kind = match &ty {
                        Type::Path(x) => {
                            let segment =
                                x.path.segments.last().ok_or_else(|| {
                                    syn::Error::new(x.span(), "Missing edge id type")
                                })?;
                            match segment.ident.to_string().to_lowercase().as_str() {
                                "option" => EntEdgeKind::Maybe,
                                "vec" => EntEdgeKind::Many,
                                _ => EntEdgeKind::One,
                            }
                        }
                        x => return Err(syn::Error::new(x.span(), "Unexpected edge id type")),
                    };

                    // Determine other properties such as deletion policy and mutability
                    for m in x.nested {
                        match m {
                            NestedMeta::Meta(x) => match x {
                                Meta::Path(x) if x.is_ident("nothing") => {
                                    deletion_policy = EntEdgeDeletionPolicy::Nothing;
                                }
                                Meta::Path(x) if x.is_ident("shallow") => {
                                    deletion_policy = EntEdgeDeletionPolicy::Shallow;
                                }
                                Meta::Path(x) if x.is_ident("deep") => {
                                    deletion_policy = EntEdgeDeletionPolicy::Deep;
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
                                        edge_type = Some(syn::parse_str::<Type>(&type_str)?);
                                    }
                                    x => {
                                        return Err(syn::Error::new(
                                            x.span(),
                                            "Unexpected edge type assignment",
                                        ))
                                    }
                                },
                                x => {
                                    return Err(syn::Error::new(
                                        x.span(),
                                        "Unexpected edge attribute",
                                    ))
                                }
                            },
                            NestedMeta::Lit(x) => {
                                return Err(syn::Error::new(
                                    x.span(),
                                    "Unexpected literal attribute",
                                ))
                            }
                        }
                    }

                    edges.push(EntEdge {
                        name,
                        ty,
                        ent_ty: edge_type
                            .ok_or_else(|| syn::Error::new(span, "Missing edge type"))?,
                        kind,
                        deletion_policy,
                    })
                }

                // For anything else, we fail because it is unsupported within ent(...)
                x => {
                    return Err(syn::Error::new(x.span(), "Unexpected ent attribute"));
                }
            }
        }

        Ok(EntInfo {
            id: id.ok_or_else(|| syn::Error::new(span, "No id field provided"))?,
            database: database
                .ok_or_else(|| syn::Error::new(span, "No database field provided"))?,
            created: created.ok_or_else(|| syn::Error::new(span, "No created field provided"))?,
            last_updated: last_updated
                .ok_or_else(|| syn::Error::new(span, "No last_updated field provided"))?,
            fields,
            edges,
        })
    }
}
