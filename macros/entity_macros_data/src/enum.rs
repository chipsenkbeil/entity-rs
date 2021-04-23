use darling::{ast, util::Flag, FromDeriveInput, FromVariant};
use syn::{Generics, Ident, Type, Visibility};

/// Information about an enum deriving ent
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ent), supports(enum_newtype))]
pub struct Ent {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: ast::Data<EntVariant, ()>,
}

/// Information for a variant of an enum deriving ent
#[derive(Debug, FromVariant)]
#[darling(attributes(ent))]
pub struct EntVariant {
    pub ident: Ident,
    pub fields: ast::Fields<Type>,
    #[darling(default)]
    pub wrap: Flag,
}
