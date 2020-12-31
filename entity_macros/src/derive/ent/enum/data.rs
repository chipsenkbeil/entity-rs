use darling::{ast, FromDeriveInput, FromVariant};
use syn::{Generics, Ident, Type, Visibility};

/// Information about an enum deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(enum_newtype))]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: ast::Data<EntVariant, ()>,

    /// Indicates not to generate a typed query struct
    #[darling(default)]
    pub no_query: bool,

    /// Indicates to include the typetag attribute on the ent trait impl,
    /// required only when Serialize/Deserialize from serde is being
    /// implemented for the given type
    #[darling(default)]
    pub typetag: bool,
}

/// Information for a variant of an enum deriving ent
#[derive(Debug, FromVariant)]
pub struct EntVariant {
    pub ident: Ident,
    pub fields: ast::Fields<Type>,
}
