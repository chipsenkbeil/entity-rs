use darling::{ast, FromDeriveInput, FromMeta, FromVariant};
use syn::{Generics, Ident, Type, Visibility};

/// Information about an enum deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(enum_tuple))]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub args: EntArgs,
    pub data: ast::Data<EntVariant, ()>,
}

/// Information for a variant of an enum deriving ent
#[derive(Debug, FromVariant)]
pub struct EntVariant {
    pub ident: Ident,
    pub fields: ast::Fields<Type>,
}

/// Enum type-level attributes for an ent
#[derive(Debug, FromMeta)]
pub struct EntArgs {
    /// Indicates not to generate a typed query struct
    pub no_query: bool,

    /// Indicates to include the typetag attribute on the ent trait impl,
    /// required only when Serialize/Deserialize from serde is being
    /// implemented for the given type
    pub typetag: bool,
}
