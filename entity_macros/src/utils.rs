use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Expr, GenericArgument, Ident, Macro, Path,
    PathArguments, PathSegment, Type,
};

/// Produces a token stream in the form of `::entity` or renamed version
pub fn entity_crate() -> darling::Result<Path> {
    crate_name("entity")
        .map(|name| {
            let crate_ident = Ident::new(&name, Span::mixed_site());
            parse_quote!(::#crate_ident)
        })
        .map_err(|msg| darling::Error::custom(msg).with_span(&Span::mixed_site()))
}

/// Produces a token stream in the form of `::serde` or renamed version
pub fn serde_crate() -> darling::Result<Path> {
    let root = entity_crate()?;
    Ok(parse_quote!(#root::vendor::macros::serde))
}

/// Produces a token stream in the form of `::typetag` or renamed version
pub fn typetag_crate() -> darling::Result<Path> {
    let root = entity_crate()?;
    Ok(parse_quote!(#root::vendor::macros::typetag))
}

/// Produces a token stream in the form of `::async_graphql` or renamed version
pub fn async_graphql_crate() -> darling::Result<Path> {
    crate_name("async-graphql")
        .map(|name| {
            let crate_ident = Ident::new(&name, Span::mixed_site());
            parse_quote!(::#crate_ident)
        })
        .map_err(|msg| darling::Error::custom(msg).with_span(&Span::mixed_site()))
}

/// Main helper called within each derive macro
pub fn do_derive(
    f: fn(Path, DeriveInput) -> darling::Result<TokenStream>,
) -> impl Fn(proc_macro::TokenStream) -> proc_macro::TokenStream {
    move |input: proc_macro::TokenStream| {
        let input = parse_macro_input!(input as DeriveInput);

        let expanded = entity_crate()
            .and_then(|root| f(root, input))
            .unwrap_or_else(|x| x.write_errors());

        proc_macro::TokenStream::from(expanded)
    }
}

/// Generates a macro call in form of `concat!(module_path!(), "::", stringify!(name))`
pub fn make_type_str(name: &Ident) -> Macro {
    parse_quote! {
        ::std::concat!(
            ::std::module_path!(),
            "::",
            ::std::stringify!(#name),
        )
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

/// Returns true if given type appears to be any of the following:
/// * [`std::collections::HashMap`]
/// * [`std::collections::BTreeMap`]
pub fn is_map_type(input: &Type) -> bool {
    type_to_ident(input)
        .map(|ident| {
            matches!(
                ident.to_string().to_lowercase().as_str(),
                "hashmap" | "btreemap"
            )
        })
        .unwrap_or_default()
}

/// Returns ident of a type if it is a type path
///
/// * `path::to::MyType` -> Some(`MyType`)
/// * `MyType` -> Some(`MyType`)
/// * `MyType<String>` -> Some(`MyType`)
pub fn type_to_ident(input: &Type) -> Option<&Ident> {
    match input {
        Type::Path(x) => match x.path.segments.last() {
            Some(x) => Some(&x.ident),
            _ => None,
        },
        _ => None,
    }
}

/// Swaps the inner type to the specified new inner value, returning a tuple
/// of (old_inner, new_full)
///
/// Option<T>, U -> (T, Option<U>)
/// Vec<T>, U -> (T, Vec<U>)
/// T, U -> (T, U)
pub fn swap_inner_type(input: &Type, new_inner: Type) -> (Type, Type) {
    if let Ok(old_inner) = strip_option(input) {
        (
            old_inner.clone(),
            parse_quote!(::std::option::Option<#new_inner>),
        )
    } else if let Ok(old_inner) = strip_vec(input) {
        (old_inner.clone(), parse_quote!(::std::vec::Vec<#new_inner>))
    } else {
        (input.clone(), new_inner)
    }
}

/// If given a type of Option<T>, will strip the outer type and return
/// a reference to type of T, returning an error if anything else
pub fn strip_option(input: &Type) -> darling::Result<&Type> {
    strip_for_type_str(input, "Option")
}

/// If given a type of Vec<T>, will strip the outer type and return
/// a reference to type of T, returning an error if anything else
pub fn strip_vec(input: &Type) -> darling::Result<&Type> {
    strip_for_type_str(input, "Vec")
}

fn strip_for_type_str<'a, 'b>(input: &'a Type, ty_str: &'b str) -> darling::Result<&'a Type> {
    match input {
        Type::Path(x) => match x.path.segments.last() {
            Some(x) if x.ident.to_string().to_lowercase() == ty_str.to_lowercase() => {
                match &x.arguments {
                    PathArguments::AngleBracketed(x) if x.args.len() == 1 => {
                        match x.args.last().unwrap() {
                            GenericArgument::Type(x) => Ok(x),
                            _ => Err(darling::Error::custom(format!(
                                "Unexpected type argument for {}",
                                ty_str
                            ))
                            .with_span(x)),
                        }
                    }
                    PathArguments::AngleBracketed(_) => Err(darling::Error::custom(format!(
                        "Unexpected number of type parameters for {}",
                        ty_str
                    ))
                    .with_span(x)),
                    PathArguments::Parenthesized(_) => Err(darling::Error::custom(format!(
                        "Unexpected {}(...) instead of {}<...>",
                        ty_str, ty_str
                    ))
                    .with_span(x)),
                    PathArguments::None => Err(darling::Error::custom(format!(
                        "{} missing generic parameter",
                        ty_str
                    ))
                    .with_span(x)),
                }
            }
            Some(x) => {
                Err(darling::Error::custom(format!("Type is not {}<...>", ty_str)).with_span(x))
            }
            None => Err(darling::Error::custom("Expected type to have a path").with_span(x)),
        },
        x => Err(darling::Error::custom("Expected type to be a path").with_span(x)),
    }
}

/// Converts some<inner<ty>> -> ty, stopping early if there is more than one
/// parameter in angle brackets such as some<inner<ty1, ty2>> -> inner<ty1, ty2>
/// and also short-circuits when encountering any other type to stop and return
pub fn get_innermost_type(ty: &Type) -> &Type {
    match ty {
        Type::Path(x) if !x.path.segments.is_empty() => {
            match &x.path.segments.last().unwrap().arguments {
                PathArguments::AngleBracketed(x) if x.args.len() == 1 => {
                    match x.args.last().unwrap() {
                        GenericArgument::Type(x) => get_innermost_type(x),
                        _ => ty,
                    }
                }
                _ => ty,
            }
        }
        _ => ty,
    }
}

/// Retrieves the inner type from a path segment, returning a reference to
/// the type at the position if available, or returning an error
pub fn get_inner_type_from_segment(
    seg: &PathSegment,
    pos: usize,
    max_supported: usize,
) -> darling::Result<&Type> {
    match &seg.arguments {
        PathArguments::AngleBracketed(x) => {
            if x.args.len() <= max_supported && x.args.len() > pos {
                match x.args.iter().nth(pos).unwrap() {
                    GenericArgument::Type(x) => Ok(x),
                    _ => Err(darling::Error::custom("Unexpected type argument").with_span(seg)),
                }
            } else {
                Err(darling::Error::custom("Invalid total type arguments").with_span(seg))
            }
        }
        _ => Err(darling::Error::custom("Unsupported type").with_span(seg)),
    }
}
