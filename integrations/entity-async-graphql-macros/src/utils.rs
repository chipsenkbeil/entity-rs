use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{
    parse_macro_input, parse_quote, DeriveInput, GenericArgument, Ident, Path, PathArguments, Type,
};

/// Produces a token stream in the form of `::entity` or renamed version
pub fn entity_crate() -> darling::Result<Path> {
    get_crate("entity")
}

/// Produces a token stream in the form of `::entity_async_graphql` or renamed version
pub fn entity_async_graphql_crate() -> darling::Result<Path> {
    get_crate("entity-async-graphql")
}

/// Produces a token stream in the form of `::async_graphql` or renamed version
pub fn async_graphql_crate() -> darling::Result<Path> {
    get_crate("async-graphql")
}

fn get_crate(cname: &str) -> darling::Result<Path> {
    crate_name(cname)
        .map(|found_crate| match found_crate {
            FoundCrate::Itself => {
                parse_quote!(crate)
            }
            FoundCrate::Name(name) => {
                let crate_ident = Ident::new(&name, Span::mixed_site());
                parse_quote!(::#crate_ident)
            }
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
