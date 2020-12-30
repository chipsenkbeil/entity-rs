use super::EntEdgeDeletionPolicy;
use darling::{ast, FromDeriveInput, FromField, FromMeta};
use syn::{spanned::Spanned, Generics, Ident, Meta, NestedMeta, Type, Visibility};

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
    pub field_attr: Option<FieldAttr>,
    #[darling(default, rename = "edge")]
    pub edge_attr: Option<EdgeAttr>,
}

/// Information for a field attribute on a field of a struct deriving ent
#[derive(Debug, Default)]
pub struct FieldAttr {
    pub indexed: bool,
    pub mutable: bool,
}

impl FromMeta for FieldAttr {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut indexed = false;
        let mut mutable = false;

        for item in items {
            match item {
                NestedMeta::Meta(x) => match x {
                    Meta::Path(x) => match x
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                        .as_str()
                    {
                        "indexed" => indexed = true,
                        "mutable" => mutable = true,
                        x => {
                            return Err(darling::Error::custom(format!("Unknown attribute: {}", x))
                                .with_span(&x.span()))
                        }
                    },
                    x => {
                        return Err(darling::Error::custom("Unsupported attribute value")
                            .with_span(&x.span()))
                    }
                },
                NestedMeta::Lit(x) => {
                    return Err(darling::Error::custom("Lit is not supported").with_span(&x.span()))
                }
            }
        }

        Ok(Self { indexed, mutable })
    }
}

/// Information for an edge attribute on a field of a struct deriving ent
#[derive(Debug, FromMeta)]
pub struct EdgeAttr {
    #[darling(rename = "type")]
    pub r#type: String,
    #[darling(multiple, rename = "wrap")]
    pub wrapping_types: Vec<String>,
    #[darling(default, rename = "policy")]
    pub deletion_policy: EntEdgeDeletionPolicy,
}
