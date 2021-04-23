use darling::{
    ast,
    util::{Flag, Override},
    FromDeriveInput, FromField, FromMeta,
};
use std::collections::HashMap;
use syn::{parse_str, Ident, TypePath};

#[derive(Debug)]
pub struct GqlEntFieldAttrMap(HashMap<Ident, GqlAttr>);

impl GqlEntFieldAttrMap {
    pub fn is_field_untyped(&self, ident: &Ident) -> bool {
        self.0
            .get(ident)
            .map_or(false, |f| f.filter_untyped.is_some())
    }

    pub fn get_field_explicit_type_str(&self, ident: &Ident) -> darling::Result<Option<TypePath>> {
        if let Some(ty) = self.0.get(ident).and_then(|f| f.filter_type.as_deref()) {
            parse_str(ty).map(Some).map_err(darling::Error::custom)
        } else {
            Ok(None)
        }
    }
}

impl From<GqlEnt> for GqlEntFieldAttrMap {
    fn from(gql_ent: GqlEnt) -> Self {
        Self(
            gql_ent
                .data
                .take_struct()
                .unwrap()
                .fields
                .into_iter()
                .filter_map(|mut f| {
                    // Build our map, collapsing field & edge attributes as a
                    // struct field can only be one but not both
                    let field = f
                        .field
                        .take()
                        .and_then(Override::explicit)
                        .and_then(|f| f.graphql);
                    let edge = f
                        .edge
                        .take()
                        .and_then(Override::explicit)
                        .and_then(|e| e.graphql);

                    f.ident.zip(field.or(edge))
                })
                .collect(),
        )
    }
}

/// Information about a struct deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(struct_named))]
pub struct GqlEnt {
    pub ident: Ident,
    pub data: ast::Data<(), GqlEntField>,
}

/// Information for a field of a struct deriving ent
#[derive(Debug, FromField)]
#[darling(allow_unknown_fields, attributes(ent))]
pub struct GqlEntField {
    pub ident: Option<Ident>,
    #[darling(default)]
    pub field: Option<Override<GqlFieldAttr>>,
    #[darling(default)]
    pub edge: Option<Override<GqlEdgeAttr>>,
}

#[derive(Debug, FromMeta)]
#[darling(allow_unknown_fields)]
pub struct GqlFieldAttr {
    #[darling(default)]
    pub graphql: Option<GqlAttr>,
}

#[derive(Debug, FromMeta)]
#[darling(allow_unknown_fields)]
pub struct GqlEdgeAttr {
    #[darling(default)]
    pub graphql: Option<GqlAttr>,
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct GqlAttr {
    #[darling(default)]
    pub filter_untyped: Flag,

    #[darling(default)]
    pub filter_type: Option<String>,
}
