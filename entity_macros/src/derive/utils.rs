use syn::{
    parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Expr, Fields, FieldsNamed,
    GenericArgument, Ident, Meta, NestedMeta, PathArguments, PathSegment, Type,
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

/// Transforms some value with the given name (ident) to the specified type,
/// producing an expression
pub fn convert_from_value(name: &Ident, ty: &Type) -> Expr {
    if let Ok(inner_ty) = strip_option(ty) {
        parse_quote! {
            #name.try_into_option::<#inner_ty>()
        }
    } else {
        parse_quote! {
            {
                use ::std::convert::TryInto;
                #name.try_into()
            }
        }
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

/// Retrieves the inner type from a path segment, returning a reference to
/// the type at the position if available, or returning an error
pub fn get_inner_type_from_segment(
    seg: &PathSegment,
    pos: usize,
    max_supported: usize,
) -> Result<&Type, syn::Error> {
    match &seg.arguments {
        PathArguments::AngleBracketed(x) => {
            if x.args.len() <= max_supported && x.args.len() > pos {
                match x.args.iter().nth(pos).unwrap() {
                    GenericArgument::Type(x) => Ok(x),
                    _ => Err(syn::Error::new(seg.span(), "Unexpected type argument")),
                }
            } else {
                Err(syn::Error::new(seg.span(), "Invalid total type arguments"))
            }
        }
        _ => Err(syn::Error::new(seg.span(), "Unsupported type")),
    }
}
