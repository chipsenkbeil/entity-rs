use syn::{
    spanned::Spanned, Attribute, Data, DeriveInput, Fields, FieldsNamed, GenericArgument, Meta,
    NestedMeta, PathArguments, Type,
};

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

/// If given a type of Option<T>, will strip the outer type and return
/// a reference to type of T, returning an error if anything else
pub fn strip_option(input: &Type) -> Result<&Type, syn::Error> {
    match input {
        Type::Path(x) => match x.path.segments.last() {
            Some(x) if x.ident.to_string().to_lowercase() == "option" => match &x.arguments {
                PathArguments::AngleBracketed(x) if x.args.len() == 1 => {
                    match x.args.last().unwrap() {
                        GenericArgument::Type(x) => Ok(x),
                        _ => Err(syn::Error::new(
                            x.span(),
                            "Unexpected type argument for Option",
                        )),
                    }
                }
                PathArguments::AngleBracketed(_) => Err(syn::Error::new(
                    x.span(),
                    "Unexpected number of type parameters for Option",
                )),
                PathArguments::Parenthesized(_) => Err(syn::Error::new(
                    x.span(),
                    "Unexpected Option(...) instead of Option<...>",
                )),
                PathArguments::None => Err(syn::Error::new(
                    x.span(),
                    "Option missing generic parameter",
                )),
            },
            Some(x) => Err(syn::Error::new(x.span(), "Type is not Option<...>")),
            None => Err(syn::Error::new(x.span(), "Expected type to have a path")),
        },
        x => Err(syn::Error::new(x.span(), "Expected type to be a path")),
    }
}
