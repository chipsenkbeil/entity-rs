use crate::utils;
use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, AttributeArgs, Field, Fields, Ident, Item, ItemStruct, NestedMeta, Path, Token,
};

pub fn do_simple_ent(root: Path, args: AttributeArgs, item: Item) -> darling::Result<TokenStream> {
    match item {
        Item::Enum(mut x) => {
            let attr_info = AttrInfo::from(&args, &x.attrs);

            inject_derive_clone_attr(&root, &mut x.attrs, &attr_info)?;
            inject_derive_ent_attr(&root, &mut x.attrs, &attr_info)?;
            inject_derive_serde_attr(&root, &mut x.attrs, &attr_info)?;

            Ok(quote! { #x })
        }
        Item::Struct(mut x) => {
            let attr_info = AttrInfo::from(&args, &x.attrs);
            let struct_info = StructInfo::from(&args, &x)?;

            inject_derive_clone_attr(&root, &mut x.attrs, &attr_info)?;
            inject_derive_ent_attr(&root, &mut x.attrs, &attr_info)?;
            inject_derive_serde_attr(&root, &mut x.attrs, &attr_info)?;
            inject_ent_id_field(&root, &mut x, &attr_info, &struct_info)?;
            inject_ent_database_field(&root, &mut x, &attr_info, &struct_info)?;
            inject_ent_created_field(&root, &mut x, &attr_info, &struct_info)?;
            inject_ent_last_updated_field(&root, &mut x, &attr_info, &struct_info)?;

            // If flagged to implement a custom debug, do so
            let debug_t = if attr_info.args_debug {
                impl_debug(&x, &attr_info.target_database_field_name)
            } else {
                quote!()
            };

            Ok(quote! {
                #x
                #debug_t
            })
        }
        x => Err(darling::Error::custom("Unsupported item").with_span(&x)),
    }
}

fn impl_debug(item: &ItemStruct, database_field_name: &str) -> TokenStream {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let debug_fields = match &item.fields {
        Fields::Named(x) => x
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref())
            .filter(|i| i != &database_field_name)
            .collect::<Vec<&Ident>>(),
        _ => unreachable!(),
    };

    quote! {
        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(::std::stringify!(#name))
                    #(.field(::std::stringify!(#debug_fields), &self.#debug_fields))*
                    .finish()
            }
        }
    }
}

/// Will add derive(Clone) to the struct if it is not already there, adding
/// a derive(...) attribute if that does not exist, either
///
/// This can be stopped via simple_ent(no_derive_clone) or forced via
/// simple_ent(derive_clone)
fn inject_derive_clone_attr(
    _root: &Path,
    attrs: &mut Vec<Attribute>,
    info: &AttrInfo,
) -> darling::Result<()> {
    if (!info.is_deriving_clone || info.args_derive_clone) && !info.args_no_derive_clone {
        let maybe_attr = attrs.iter_mut().find(|a| {
            a.path
                .segments
                .last()
                .filter(|s| s.ident == "derive")
                .is_some()
        });

        // If we already have a derive, we want to insert ourselves into it
        if let Some(attr) = maybe_attr {
            let mut args =
                attr.parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?;
            args.push(parse_quote!(::std::clone::Clone));
            attr.tokens = quote!((#args));

        // Otherwise, we need to create the derive from scratch
        } else {
            let mut tmp_attrs: ParsableOuterAttributes =
                parse_quote!(#[derive(::std::clone::Clone)]);
            let derive_attr = tmp_attrs.attributes.pop().unwrap();
            attrs.push(derive_attr);
        }
    }

    Ok(())
}

/// Will add derive(Ent) to the struct if it is not already there, adding
/// a derive(...) attribute if that does not exist, either
///
/// This can be stopped via simple_ent(no_derive_ent) or forced via
/// simple_ent(derive_ent)
fn inject_derive_ent_attr(
    root: &Path,
    attrs: &mut Vec<Attribute>,
    info: &AttrInfo,
) -> darling::Result<()> {
    if (!info.is_deriving_ent || info.args_derive_ent) && !info.args_no_derive_ent {
        let maybe_attr = attrs.iter_mut().find(|a| {
            a.path
                .segments
                .last()
                .filter(|s| s.ident == "derive")
                .is_some()
        });

        // If we already have a derive, we want to insert ourselves into it
        if let Some(attr) = maybe_attr {
            let mut args =
                attr.parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?;
            args.push(parse_quote!(#root::Ent));
            attr.tokens = quote!((#args));

        // Otherwise, we need to create the derive from scratch
        } else {
            let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[derive(#root::Ent)]);
            let derive_attr = tmp_attrs.attributes.pop().unwrap();
            attrs.push(derive_attr);
        }
    }

    Ok(())
}

/// Will add derive(Serialize, Deserialize) to the struct if it is not already
/// there, adding a derive(...) attribute if that does not exist, either
///
/// This is only done if specified via simple_ent(serde) and can be prevented
/// via simple_ent(no_serde)
fn inject_derive_serde_attr(
    _root: &Path,
    attrs: &mut Vec<Attribute>,
    info: &AttrInfo,
) -> darling::Result<()> {
    if !info.args_no_serde {
        let maybe_attr = attrs.iter_mut().find(|a| {
            a.path
                .segments
                .last()
                .filter(|s| s.ident == "derive")
                .is_some()
        });
        let serde_root = utils::serde_crate()?;

        if !info.is_deriving_serialize || !info.is_deriving_deserialize {
            let new_attrs = match (info.is_deriving_serialize, info.is_deriving_deserialize) {
                (false, false) => vec![
                    quote!(#serde_root::Serialize),
                    quote!(#serde_root::Deserialize),
                ],
                (false, true) => vec![quote!(#serde_root::Serialize)],
                (true, false) => vec![quote!(#serde_root::Deserialize)],
                (true, true) => unreachable!(),
            };

            // If we already have a derive, we want to insert ourselves into it
            if let Some(attr) = maybe_attr {
                let mut args =
                    attr.parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?;
                for new_attr in new_attrs {
                    args.push(parse_quote!(#new_attr));
                }
                attr.tokens = quote!((#args));

            // Otherwise, we need to create the derive from scratch
            } else {
                let mut tmp_attrs: ParsableOuterAttributes =
                    parse_quote!(#[derive(#(#new_attrs),*)]);
                let derive_attr = tmp_attrs.attributes.pop().unwrap();
                attrs.push(derive_attr);
            }
        }
    }

    Ok(())
}

/// Will add a named field to the struct in the form of `id: entity::Id` that
/// has `#[ent(id)]` included. This is only done if another field is not
/// already marked as the id. Will fail if a field already exists with the
/// id's name and there is no marked id field.
///
/// The name can be altered via simple_ent(id = "...")
fn inject_ent_id_field(
    root: &Path,
    item: &mut ItemStruct,
    attr_info: &AttrInfo,
    struct_info: &StructInfo,
) -> darling::Result<()> {
    match (
        struct_info.has_conflicting_id_name,
        struct_info.has_id_marker,
    ) {
        (Some(span), None) => Err(darling::Error::custom("Conflicting field with same name")
            .with_span(&SpannedValue::new((), span))),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_id_field_name);
                    let named_field: ParsableNamedField = parse_quote! {
                        #[ent(id)]
                        #name: #root::Id
                    };

                    x.named.push(named_field.field);
                }

                // By nature of having Info, we have already verified that we have
                // named fields
                _ => unreachable!(),
            }

            Ok(())
        }
    }
}

/// Will add a named field to the struct in the form of
/// `database: WeakDatabaseRc` that has `#[ent(database)]`
/// included. This is only done if another field is not already marked as the
/// database. Will fail if a field already exists with the database's name and
/// there is no marked database field.
///
/// Additionally, the field will include `#[serde(skip)]` if it is detected
/// that `serde::Serialize` or `serde::Deserialize`.
///
/// The name can be altered via simple_ent(database = "...")
fn inject_ent_database_field(
    root: &Path,
    item: &mut ItemStruct,
    attr_info: &AttrInfo,
    struct_info: &StructInfo,
) -> darling::Result<()> {
    match (
        struct_info.has_conflicting_database_name,
        struct_info.has_database_marker,
    ) {
        (Some(span), None) => Err(darling::Error::custom("Conflicting field with same name")
            .with_span(&SpannedValue::new((), span))),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_database_field_name);

                    // If we are deriving serde, we need to mark the database
                    // as skippable as it cannot be serialized. This can
                    // be forced excluded via simple_ent(no_serde)
                    let skip_attr = if !attr_info.args_no_serde {
                        quote! { #[serde(skip)] }
                    } else {
                        quote! {}
                    };

                    let named_field: ParsableNamedField = parse_quote! {
                        #skip_attr
                        #[ent(database)]
                        #name: #root::WeakDatabaseRc
                    };

                    x.named.push(named_field.field);
                }

                // By nature of having Info, we have already verified that we have
                // named fields
                _ => unreachable!(),
            }

            Ok(())
        }
    }
}

/// Will add a named field to the struct in the form of `created: u64` that
/// has `#[ent(created)]` included. This is only done if another field is not
/// already marked as the created. Will fail if a field already exists with the
/// created's name and there is no marked created field.
///
/// The name can be altered via simple_ent(created = "...")
fn inject_ent_created_field(
    _root: &Path,
    item: &mut ItemStruct,
    attr_info: &AttrInfo,
    struct_info: &StructInfo,
) -> darling::Result<()> {
    match (
        struct_info.has_conflicting_created_name,
        struct_info.has_created_marker,
    ) {
        (Some(span), None) => Err(darling::Error::custom("Conflicting field with same name")
            .with_span(&SpannedValue::new((), span))),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_created_field_name);
                    let named_field: ParsableNamedField = parse_quote! {
                        #[ent(created)]
                        #name: ::std::primitive::u64
                    };

                    x.named.push(named_field.field);
                }

                // By nature of having Info, we have already verified that we have
                // named fields
                _ => unreachable!(),
            }

            Ok(())
        }
    }
}

/// Will add a named field to the struct in the form of `last_updated: u64`
/// that has `#[ent(last_updated)]` included. This is only done if another
/// field is not already marked as the last_updated. Will fail if a field
/// already exists with the last_updated's name and there is no marked
/// last_updated field.
///
/// The name can be altered via simple_ent(last_updated = "...")
fn inject_ent_last_updated_field(
    _root: &Path,
    item: &mut ItemStruct,
    attr_info: &AttrInfo,
    struct_info: &StructInfo,
) -> darling::Result<()> {
    match (
        struct_info.has_conflicting_last_updated_name,
        struct_info.has_last_updated_marker,
    ) {
        (Some(span), None) => Err(darling::Error::custom("Conflicting field with same name")
            .with_span(&SpannedValue::new((), span))),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_last_updated_field_name);
                    let named_field: ParsableNamedField = parse_quote! {
                        #[ent(last_updated)]
                        #name: ::std::primitive::u64
                    };

                    x.named.push(named_field.field);
                }

                // By nature of having Info, we have already verified that we have
                // named fields
                _ => unreachable!(),
            }

            Ok(())
        }
    }
}

/// Information collected from examining our attribute args
#[derive(Default, Debug)]
struct AttrInfo {
    is_deriving_clone: bool,
    is_deriving_ent: bool,
    is_deriving_serialize: bool,
    is_deriving_deserialize: bool,
    args_no_serde: bool,
    args_no_derive_ent: bool,
    args_derive_ent: bool,
    args_no_derive_clone: bool,
    args_derive_clone: bool,
    args_debug: bool,
    target_id_field_name: String,
    target_database_field_name: String,
    target_created_field_name: String,
    target_last_updated_field_name: String,
}

impl AttrInfo {
    #[allow(clippy::field_reassign_with_default)]
    pub fn from(args: &[NestedMeta], attrs: &[Attribute]) -> Self {
        let mut args_attrs = utils::nested_meta_iter_into_named_attr_map(args.iter());
        let derive_attrs = utils::attrs_into_attr_map(attrs, "derive").unwrap_or_default();

        let mut info = AttrInfo::default();

        info.is_deriving_clone = derive_attrs.get("Clone").copied().unwrap_or_default();
        info.is_deriving_ent = derive_attrs.get("Ent").copied().unwrap_or_default();
        info.is_deriving_serialize = derive_attrs.get("Serialize").copied().unwrap_or_default();
        info.is_deriving_deserialize = derive_attrs.get("Deserialize").copied().unwrap_or_default();

        info.target_id_field_name = args_attrs
            .remove("id")
            .flatten()
            .unwrap_or_else(|| String::from("id"));
        info.target_database_field_name = args_attrs
            .remove("database")
            .flatten()
            .unwrap_or_else(|| String::from("database"));
        info.target_created_field_name = args_attrs
            .remove("created")
            .flatten()
            .unwrap_or_else(|| String::from("created"));
        info.target_last_updated_field_name = args_attrs
            .remove("last_updated")
            .flatten()
            .unwrap_or_else(|| String::from("last_updated"));

        info.args_no_serde = args_attrs.contains_key("no_serde");
        info.args_no_derive_ent = args_attrs.contains_key("no_derive_ent");
        info.args_derive_ent = args_attrs.contains_key("derive_ent");
        info.args_no_derive_clone = args_attrs.contains_key("no_derive_clone");
        info.args_derive_clone = args_attrs.contains_key("derive_clone");
        info.args_debug = args_attrs.contains_key("debug");

        info
    }
}

/// Information collected from examining the item struct and our attribute args
#[derive(Default, Debug)]
struct StructInfo {
    has_id_marker: Option<Span>,
    has_database_marker: Option<Span>,
    has_created_marker: Option<Span>,
    has_last_updated_marker: Option<Span>,
    has_conflicting_id_name: Option<Span>,
    has_conflicting_database_name: Option<Span>,
    has_conflicting_created_name: Option<Span>,
    has_conflicting_last_updated_name: Option<Span>,
}

impl StructInfo {
    pub fn from(args: &[NestedMeta], input: &ItemStruct) -> darling::Result<Self> {
        let attr_info = AttrInfo::from(args, &input.attrs);
        let mut info = StructInfo::default();

        match &input.fields {
            Fields::Named(x) => {
                for f in x.named.iter() {
                    let name = f.ident.as_ref().unwrap().to_string();

                    // Check if the field has a conflicting name
                    if name == attr_info.target_id_field_name {
                        info.has_conflicting_id_name = Some(f.ident.span());
                    } else if name == attr_info.target_database_field_name {
                        info.has_conflicting_database_name = Some(f.ident.span());
                    } else if name == attr_info.target_created_field_name {
                        info.has_conflicting_created_name = Some(f.ident.span());
                    } else if name == attr_info.target_last_updated_field_name {
                        info.has_conflicting_last_updated_name = Some(f.ident.span());
                    }

                    // Check if the field is one of our markers
                    if utils::has_ent_attr(&f.attrs, "id") {
                        info.has_id_marker = Some(f.span());
                    } else if utils::has_ent_attr(&f.attrs, "database") {
                        info.has_database_marker = Some(f.span());
                    } else if utils::has_ent_attr(&f.attrs, "created") {
                        info.has_created_marker = Some(f.span());
                    } else if utils::has_ent_attr(&f.attrs, "last_updated") {
                        info.has_last_updated_marker = Some(f.span());
                    }
                }
            }
            Fields::Unnamed(_) => {
                return Err(darling::Error::custom("Tuple struct not supported").with_span(input))
            }
            Fields::Unit => {
                return Err(darling::Error::custom("Unit struct not supported").with_span(input))
            }
        }

        Ok(info)
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
