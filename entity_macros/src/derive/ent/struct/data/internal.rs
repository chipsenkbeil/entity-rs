use super::EntEdgeDeletionPolicy;
use darling::{ast, util::Override, FromDeriveInput, FromField, FromMeta};
use syn::{Generics, Ident, Type, Visibility};

/// Information about a struct deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(struct_named))]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: ast::Data<(), EntField>,
    #[darling(default)]
    pub no_builder: bool,
    #[darling(default)]
    pub no_query: bool,
    #[darling(default)]
    pub no_typed_methods: bool,
    #[darling(default)]
    pub typetag: bool,
    #[darling(default)]
    pub strict: bool,
}

/// Information for a field of a struct deriving ent
#[derive(Debug, FromField)]
#[darling(attributes(ent))]
pub struct EntField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default, rename = "id")]
    pub is_ent_id_field: bool,
    #[darling(default, rename = "database")]
    pub is_ent_database_field: bool,
    #[darling(default, rename = "created")]
    pub is_ent_created_field: bool,
    #[darling(default, rename = "last_updated")]
    pub is_ent_last_updated_field: bool,
    #[darling(default, rename = "field")]
    pub field_attr: Option<Override<FieldAttr>>,
    #[darling(default, rename = "edge")]
    pub edge_attr: Option<EdgeAttr>,
}

/// Information for a field attribute on a field of a struct deriving ent
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct FieldAttr {
    pub indexed: bool,
    pub mutable: bool,
}

/// Information for an edge attribute on a field of a struct deriving ent
#[derive(Debug, FromMeta)]
pub struct EdgeAttr {
    #[darling(rename = "type")]
    pub r#type: String,
    #[darling(default)]
    pub wrap: bool,
    #[darling(default, rename = "policy")]
    pub deletion_policy: EntEdgeDeletionPolicy,
}
