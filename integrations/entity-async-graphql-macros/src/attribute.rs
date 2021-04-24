use crate::utils;
use darling::{
    ast,
    util::{Ignored, PathList},
    FromDeriveInput, FromMeta,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote, Attribute, AttributeArgs, DeriveInput, Field, Generics, Ident, Path, Visibility,
};

#[derive(Debug, FromDeriveInput)]
#[darling(
    allow_unknown_fields,
    supports(struct_named, enum_newtype),
    forward_attrs(derive, simple_ent)
)]
struct Ent {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    data: ast::Data<Ignored, Ignored>,
    attrs: Vec<Attribute>,
}

#[derive(Debug, Default)]
struct EntDerive {
    async_graphql_union: bool,
    ent_object: bool,
    ent_filter: bool,
}

impl From<PathList> for EntDerive {
    fn from(pl: PathList) -> Self {
        pl.iter().fold(Self::default(), |mut d, path| {
            match path
                .segments
                .last()
                .unwrap()
                .ident
                .to_string()
                .to_lowercase()
                .as_str()
            {
                "union" => d.async_graphql_union = true,
                "entobject" => d.ent_object = true,
                "entfilter" => d.ent_filter = true,
                _ => {}
            }

            d
        })
    }
}

pub fn do_gql_ent(
    root: Path,
    _args: AttributeArgs,
    mut input: DeriveInput,
) -> darling::Result<TokenStream> {
    let ent = Ent::from_derive_input(&input)?;
    let has_simple_ent = ent.attrs.iter().any(|a| {
        a.parse_meta()
            .ok()
            .as_ref()
            .filter(|x| x.path().is_ident("simple_ent"))
            .is_some()
    });
    if !has_simple_ent {
        input.attrs.insert(0, make_simple_ent_attr(&root));
    }

    let ent_derive = ent
        .attrs
        .iter()
        .find_map(|a| {
            a.parse_meta()
                .ok()
                .as_ref()
                .filter(|x| x.path().is_ident("derive"))
                .and_then(|x| PathList::from_meta(x).ok())
                .map(EntDerive::from)
        })
        .unwrap_or_default();

    let mut derive_paths: Vec<Path> = Vec::new();

    match ent.data {
        ast::Data::Enum(_) => {
            if !ent_derive.async_graphql_union {
                let async_graphql_root = utils::async_graphql_crate()?;
                derive_paths.push(parse_quote!(#async_graphql_root::Union));
            }

            if !derive_paths.is_empty() {
                input.attrs.push(make_derive_attr(derive_paths));
            }

            Ok(quote!(#input))
        }
        ast::Data::Struct(_) => {
            if !ent_derive.ent_object {
                let entity_async_graphql_root = utils::entity_async_graphql_crate()?;
                derive_paths.push(parse_quote!(#entity_async_graphql_root::EntObject));
            }

            if !ent_derive.ent_filter {
                let entity_async_graphql_root = utils::entity_async_graphql_crate()?;
                derive_paths.push(parse_quote!(#entity_async_graphql_root::EntFilter));
            }

            if !derive_paths.is_empty() {
                input.attrs.push(make_derive_attr(derive_paths));
            }

            Ok(quote!(#input))
        }
    }
}

/// Produces #[simple_ent]
fn make_simple_ent_attr(root: &Path) -> Attribute {
    let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[#root::simple_ent]);
    tmp_attrs.attributes.pop().unwrap()
}

/// Translates vec![Path1, Path2, ...] -> derive(Path1, Path2, ...)
fn make_derive_attr(paths: Vec<Path>) -> Attribute {
    let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[derive(#(#paths),*)]);
    tmp_attrs.attributes.pop().unwrap()
}

/// Workaround to parse a field using parse_quote! as described here:
/// https://github.com/dtolnay/syn/issues/651#issuecomment-503771863
struct ParsableNamedField {
    pub field: Field,
}

impl Parse for ParsableNamedField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let field = Field::parse_named(input)?;

        Ok(ParsableNamedField { field })
    }
}

struct ParsableOuterAttributes {
    pub attributes: Vec<Attribute>,
}

impl Parse for ParsableOuterAttributes {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let attributes = Attribute::parse_outer(input)?;

        Ok(ParsableOuterAttributes { attributes })
    }
}
