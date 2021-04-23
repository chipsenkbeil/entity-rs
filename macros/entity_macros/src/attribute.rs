use crate::utils;
use darling::{
    ast,
    util::{Flag, Ignored, PathList},
    FromDeriveInput, FromField, FromMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    Attribute, AttributeArgs, DeriveInput, Field, Generics, Ident, LitStr, Meta, NestedMeta, Path,
    Type, Visibility,
};

const DEFAULT_ID_NAME: &str = "id";
const DEFAULT_DATABASE_NAME: &str = "database";
const DEFAULT_CREATED_NAME: &str = "created";
const DEFAULT_LAST_UPDATED_NAME: &str = "last_updated";

#[derive(Debug, FromDeriveInput)]
#[darling(
    allow_unknown_fields,
    supports(struct_named, enum_newtype),
    forward_attrs(derive)
)]
struct Ent {
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    data: ast::Data<Ignored, EntField>,
    attrs: Vec<Attribute>,
}

#[derive(Debug, FromMeta)]
struct EntArgs {
    #[darling(default)]
    id: Option<String>,
    #[darling(default)]
    database: Option<String>,
    #[darling(default)]
    created: Option<String>,
    #[darling(default)]
    last_updated: Option<String>,
}

impl EntArgs {
    fn id_ident(&self) -> Ident {
        format_ident!("{}", self.id.as_deref().unwrap_or(DEFAULT_ID_NAME))
    }

    fn database_ident(&self) -> Ident {
        format_ident!(
            "{}",
            self.database.as_deref().unwrap_or(DEFAULT_DATABASE_NAME)
        )
    }

    fn created_ident(&self) -> Ident {
        format_ident!(
            "{}",
            self.created.as_deref().unwrap_or(DEFAULT_CREATED_NAME)
        )
    }

    fn last_updated_ident(&self) -> Ident {
        format_ident!(
            "{}",
            self.last_updated
                .as_deref()
                .unwrap_or(DEFAULT_LAST_UPDATED_NAME)
        )
    }
}

#[derive(Debug, Default)]
struct EntDerive {
    clone: bool,
    serde_serialize: bool,
    serde_deserialize: bool,
    ent: bool,
    ent_debug: bool,
    ent_type: bool,
    ent_builder: bool,
    ent_loader: bool,
    ent_query: bool,
    ent_typed_fields: bool,
    ent_typed_edges: bool,
    ent_wrapper: bool,
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
                "clone" => d.clone = true,
                "serialize" => d.serde_serialize = true,
                "deserialize" => d.serde_deserialize = true,
                "ent" => d.ent = true,
                "entdebug" => d.ent_debug = true,
                "enttype" => d.ent_type = true,
                "entbuilder" => d.ent_builder = true,
                "entloader" => d.ent_loader = true,
                "entquery" => d.ent_query = true,
                "enttypedfields" => d.ent_typed_fields = true,
                "enttypededges" => d.ent_typed_edges = true,
                "entwrapper" => d.ent_wrapper = true,
                _ => {}
            }

            d
        })
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(ent), allow_unknown_fields)]
struct EntField {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    id: Flag,

    #[darling(default)]
    database: Flag,

    #[darling(default)]
    created: Flag,

    #[darling(default)]
    last_updated: Flag,
}

pub fn do_simple_ent(
    root: Path,
    args: AttributeArgs,
    mut input: DeriveInput,
) -> darling::Result<TokenStream> {
    let ent_args = EntArgs::from_list(&args)?;
    let ent = Ent::from_derive_input(&input)?;
    let ent_derive = ent
        .attrs
        .iter()
        .find_map(|a| {
            a.parse_meta()
                .ok()
                .as_ref()
                .and_then(|x| PathList::from_meta(x).ok())
                .map(EntDerive::from)
        })
        .unwrap_or_default();

    let mut derive_paths: Vec<Path> = Vec::new();

    if !ent_derive.clone {
        derive_paths.push(parse_quote!(::std::clone::Clone));
    }

    if !ent_derive.serde_serialize {
        let serde_root = utils::serde_crate()?;
        derive_paths.push(parse_quote!(#serde_root::Serialize));
    }

    if !ent_derive.serde_deserialize {
        let serde_root = utils::serde_crate()?;
        derive_paths.push(parse_quote!(#serde_root::Deserialize));
    }

    if !ent_derive.ent {
        derive_paths.push(parse_quote!(#root::Ent));
    }

    if !ent_derive.ent_type {
        derive_paths.push(parse_quote!(#root::EntType));
    }

    if !ent_derive.ent_query {
        derive_paths.push(parse_quote!(#root::EntQuery));
    }

    match ent.data {
        ast::Data::Enum(_) => {
            if !ent_derive.ent_wrapper {
                derive_paths.push(parse_quote!(#root::EntWrapper));
            }

            if !derive_paths.is_empty() {
                input.attrs.push(make_derive_attr(derive_paths));
            }

            Ok(quote!(#input))
        }
        ast::Data::Struct(fields) => {
            if !ent_derive.ent_builder {
                derive_paths.push(parse_quote!(#root::EntBuilder));
            }

            if !ent_derive.ent_loader {
                derive_paths.push(parse_quote!(#root::EntLoader));
            }

            if !ent_derive.ent_typed_fields {
                derive_paths.push(parse_quote!(#root::EntTypedFields));
            }

            if !ent_derive.ent_typed_edges {
                derive_paths.push(parse_quote!(#root::EntTypedEdges));
            }

            if !ent_derive.ent_debug {
                derive_paths.push(parse_quote!(#root::EntDebug));
            }

            if !derive_paths.is_empty() {
                input.attrs.push(make_derive_attr(derive_paths));
            }

            // Determine which of the required fields we will inject and add them
            match &mut input.data {
                syn::Data::Struct(x) => match &mut x.fields {
                    syn::Fields::Named(x) => {
                        for f in x.named.iter_mut() {
                            modify_field_if_computed_field(&root, f);
                            modify_field_if_edge(&root, f);
                        }

                        if !fields.iter().any(|f| f.id.is_some()) {
                            x.named.push(make_field(
                                ent_args.id_ident(),
                                parse_quote!(#root::Id),
                                quote!(#[ent(id)]),
                            ));
                        }

                        if !fields.iter().any(|f| f.database.is_some()) {
                            x.named.push(make_field(
                                ent_args.database_ident(),
                                parse_quote!(#root::WeakDatabaseRc),
                                quote! {
                                    #[ent(database)]
                                    #[serde(skip)]
                                },
                            ));
                        }

                        if !fields.iter().any(|f| f.created.is_some()) {
                            x.named.push(make_field(
                                ent_args.created_ident(),
                                parse_quote!(::std::primitive::u64),
                                quote!(#[ent(created)]),
                            ));
                        }

                        if !fields.iter().any(|f| f.last_updated.is_some()) {
                            x.named.push(make_field(
                                ent_args.last_updated_ident(),
                                parse_quote!(::std::primitive::u64),
                                quote!(#[ent(last_updated)]),
                            ));
                        }
                    }
                    _ => {
                        return Err(darling::Error::custom("Only named structs are supported")
                            .with_span(&input.span()))
                    }
                },
                _ => unreachable!(),
            }

            Ok(quote!(#input))
        }
    }
}

/// Translates vec![Path1, Path2, ...] -> derive(Path1, Path2, ...)
fn make_derive_attr(paths: Vec<Path>) -> Attribute {
    let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[derive(#(#paths),*)]);
    tmp_attrs.attributes.pop().unwrap()
}

fn make_field(name: Ident, ty: Type, attrs: TokenStream) -> Field {
    let named_field: ParsableNamedField = parse_quote! {
        #attrs
        #name: #ty
    };

    named_field.field
}

/// Modify any field marked as a field that is computed to transform it
/// from
///
/// #[ent(field(computed = "..."))]
/// field1: u32
///
/// into
///
/// #[ent(field(computed = "..."))]
/// field1: Option<u32>
fn modify_field_if_computed_field(_root: &Path, field: &mut Field) {
    if let Some(ent_attr) = field.attrs.iter_mut().find(|a| a.path.is_ident("ent")) {
        if let Ok(Meta::List(x)) = ent_attr.parse_meta() {
            let (mut ent_field, _): (Vec<&NestedMeta>, Vec<&NestedMeta>) =
                x.nested.iter().partition(|&nm| match nm {
                    NestedMeta::Meta(x) => x.path().is_ident("field"),
                    _ => false,
                });

            if ent_field.len() == 1 {
                if let NestedMeta::Meta(Meta::List(x)) = ent_field.pop().unwrap() {
                    let has_computed_attr = x.nested.iter().any(|nm| match nm {
                        NestedMeta::Meta(Meta::NameValue(x)) => x.path.is_ident("computed"),
                        _ => false,
                    });

                    // We have ent(field(computed = "...", ...)), so we
                    // can transform the type from ty -> Option<ty>
                    if has_computed_attr {
                        let ty = &field.ty;
                        field.ty = parse_quote!(::std::option::Option<#ty>);
                    }
                }
            }
        }
    }
}

/// Modify any field marked as an edge without a type
/// as having the type equal to the current field type
/// and replace the field type with id
///
/// e.g.
///
/// #[ent(edge)]             --> #[ent(edge(type = "MyEnt"))]
/// my_edge: MyEnt           --> my_edge: Id,
///
/// #[ent(edge)]             --> #[ent(edge(type = "MyEnt"))]
/// my_edge: Option<MyEnt>   --> my_edge: Option<Id>,
///
/// #[ent(edge)]             --> #[ent(edge(type = "MyEnt"))]
/// my_edge: Vec<MyEnt>      --> my_edge: Vec<Id>,
fn modify_field_if_edge(root: &Path, field: &mut Field) {
    if let Some(ent_attr) = field.attrs.iter_mut().find(|a| a.path.is_ident("ent")) {
        if let Ok(Meta::List(x)) = ent_attr.parse_meta() {
            let (mut edge, not_edge): (Vec<&NestedMeta>, Vec<&NestedMeta>) =
                x.nested.iter().partition(|&nm| match nm {
                    NestedMeta::Meta(x) => x.path().is_ident("edge"),
                    _ => false,
                });

            if edge.len() == 1 {
                if let NestedMeta::Meta(x) = edge.pop().unwrap() {
                    match x {
                        Meta::Path(_) => {
                            let (ent_ty, ty) =
                                utils::swap_inner_type(&field.ty, parse_quote!(#root::Id));
                            let lit_ty =
                                LitStr::new(&format!("{}", quote!(#ent_ty)), Span::mixed_site());
                            field.ty = ty;
                            ent_attr.tokens = quote! {
                                (edge(type = #lit_ty), #(#not_edge),*)
                            };
                        }
                        Meta::List(x) => {
                            let (r#type, tail): (Vec<&NestedMeta>, Vec<&NestedMeta>) =
                                x.nested.iter().partition(|&nm| match nm {
                                    NestedMeta::Meta(Meta::NameValue(x)) => x.path.is_ident("type"),
                                    _ => false,
                                });

                            // We have ent(edge(...)) without type = "..."
                            if r#type.is_empty() {
                                let (ent_ty, ty) =
                                    utils::swap_inner_type(&field.ty, parse_quote!(#root::Id));
                                let lit_ty = LitStr::new(
                                    &format!("{}", quote!(#ent_ty)),
                                    Span::mixed_site(),
                                );
                                field.ty = ty;
                                ent_attr.tokens = quote! {
                                    (edge(type = #lit_ty, #(#tail),*), #(#not_edge),*)
                                };
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
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
