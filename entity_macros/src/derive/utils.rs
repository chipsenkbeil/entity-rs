use syn::{spanned::Spanned, Attribute, Data, DeriveInput, Fields, FieldsNamed, Meta, NestedMeta};

/// Returns true if the attribute is in the form of ent(...) where
/// the interior is checked for an identifier of the given str
pub fn has_outer_ent_attr(attrs: &[Attribute], ident_str: &str) -> bool {
    attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .any(|m| match m {
            Meta::List(x) if x.path.is_ident("ent") => x.nested.iter().any(|m| match m {
                NestedMeta::Meta(x) => match x {
                    Meta::Path(x) => x.is_ident(ident_str),
                    _ => false,
                },
                _ => false,
            }),
            _ => false,
        })
}

/// Extracts and returns the named fields from the input, if possible
pub fn get_named_fields(input: &DeriveInput) -> Result<&FieldsNamed, syn::Error> {
    match &input.data {
        Data::Struct(x) => match &x.fields {
            Fields::Named(x) => Ok(x),
            _ => Err(syn::Error::new(input.span(), "Expected named fields")),
        },
        _ => Err(syn::Error::new(input.span(), "Expected struct")),
    }
}
