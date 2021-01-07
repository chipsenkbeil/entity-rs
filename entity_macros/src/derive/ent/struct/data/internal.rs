use super::EntEdgeDeletionPolicy;
use darling::{
    ast,
    util::{Override, SpannedValue},
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
    #[darling(default)]
    pub id: Option<SpannedValue<()>>,
    #[darling(default)]
    pub database: Option<SpannedValue<()>>,
    #[darling(default)]
    pub created: Option<SpannedValue<()>>,
    #[darling(default)]
    pub last_updated: Option<SpannedValue<()>>,
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
            for item in vec![&self.id, &self.database, &self.created, &self.last_updated] {
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
#[darling(default)]
pub struct FieldAttr {
    pub indexed: bool,
    pub mutable: bool,
}

/// Information for an edge attribute on a field of a struct deriving ent
#[derive(Debug, Clone, FromMeta)]
pub struct EdgeAttr {
    #[darling(rename = "type")]
    pub r#type: String,
    #[darling(default)]
    pub wrap: bool,
    #[darling(default, rename = "policy")]
    pub deletion_policy: EntEdgeDeletionPolicy,
}
