use super::EntEdgeDeletionPolicy;
use darling::{
    ast,
    util::{Flag, Override, SpannedValue},
    FromDeriveInput, FromField, FromMeta,
};
use syn::{Generics, Ident, Type, Visibility};

/// Information about a struct deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(struct_named))]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: ast::Data<(), EntField>,
}

/// Information for a field of a struct deriving ent
#[derive(Debug, FromField)]
#[darling(attributes(ent))]
pub struct EntField {
    pub ident: Option<Ident>,
    pub ty: Type,
    /// Location of the word `id`, if present.
    #[darling(default)]
    id: Option<SpannedValue<()>>,
    /// Location of the word `database`, if present.
    #[darling(default)]
    database: Option<SpannedValue<()>>,
    /// Location of the word `created`, if present.
    #[darling(default)]
    created: Option<SpannedValue<()>>,
    /// Location of the word `last_updated`, if present.
    #[darling(default)]
    last_updated: Option<SpannedValue<()>>,
    #[darling(default, rename = "field")]
    pub field_attr: Option<Override<FieldAttr>>,
    #[darling(default, rename = "edge")]
    pub edge_attr: Option<EdgeAttr>,
}

impl EntField {
    pub fn is_id_field(&self) -> bool {
        self.id.is_some()
    }

    pub fn is_database_field(&self) -> bool {
        self.database.is_some()
    }

    pub fn is_created_field(&self) -> bool {
        self.created.is_some()
    }

    pub fn is_last_updated_field(&self) -> bool {
        self.last_updated.is_some()
    }

    /// Check whether this field is trying to declare itself as multiple ent fields.
    /// This is nonsensical, and indicates an error on the part of the caller.
    ///
    /// # Example
    /// This would return true for `#[ent(id, database)]`.
    fn declares_multiple_known_fields(&self) -> bool {
        let mut known_fields_declared = 0;
        if self.is_id_field() {
            known_fields_declared += 1;
        }

        if self.is_database_field() {
            known_fields_declared += 1;
        }

        if self.is_created_field() {
            known_fields_declared += 1;
        }

        if self.is_last_updated_field() {
            known_fields_declared += 1;
        }

        known_fields_declared > 1
    }

    /// Validate that at most one of `id`, `database`, `created`, and `last_updated` are declared.
    /// If that is not the case, create properly-spanned errors for each declaration.
    pub fn validate_zero_or_one_known_fields(&self) -> darling::Result<()> {
        if self.declares_multiple_known_fields() {
            let mut errors = vec![];
            for item in &[&self.id, &self.database, &self.created, &self.last_updated] {
                if let Some(word) = item {
                    errors.push(darling::Error::custom("A field can only declare one of `id`, `database`, `created`, or `last_updated`").with_span(word));
                }
            }

            Err(darling::Error::multiple(errors))
        } else {
            Ok(())
        }
    }
}

/// Information for a field attribute on a field of a struct deriving ent
#[derive(Debug, Clone, Default, FromMeta)]
#[darling(allow_unknown_fields, default)]
pub struct FieldAttr {
    #[darling(default)]
    pub indexed: Flag,
    #[darling(default)]
    pub mutable: Flag,
}

/// Information for an edge attribute on a field of a struct deriving ent
#[derive(Debug, Clone, FromMeta)]
#[darling(allow_unknown_fields)]
pub struct EdgeAttr {
    #[darling(rename = "type")]
    pub r#type: String,
    #[darling(default, rename = "query_type")]
    pub query_ty: Option<String>,
    #[darling(default)]
    pub wrap: Flag,
    #[darling(default)]
    pub use_id_slice: Flag,
    #[darling(default, rename = "policy")]
    pub deletion_policy: EntEdgeDeletionPolicy,
}

/// Information specifically related to the `async-graphql` extension
#[derive(Clone, Debug, Default, FromMeta)]
pub struct AsyncGraphqlExtAttr {
    #[darling(default)]
    pub filter_untyped: Flag,

    #[darling(default)]
    pub filter_type: Option<String>,
}
