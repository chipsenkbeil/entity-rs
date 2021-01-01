use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, AttributeArgs, Field, Fields, Item, ItemStruct, NestedMeta, Path, Token,
};

pub fn do_simple_ent(
    root: Path,
    args: AttributeArgs,
    item: Item,
) -> Result<TokenStream, syn::Error> {
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

            Ok(quote! { #x })
        }
        x => Err(syn::Error::new(x.span(), "Unsupported item")),
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
) -> Result<(), syn::Error> {
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
) -> Result<(), syn::Error> {
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
) -> Result<(), syn::Error> {
    if info.args_serde && !info.args_no_serde {
        let maybe_attr = attrs.iter_mut().find(|a| {
            a.path
                .segments
                .last()
                .filter(|s| s.ident == "derive")
                .is_some()
        });
        let serde_root = utils::serde_crate()?;
        let new_attr = match (info.is_deriving_serialize, info.is_deriving_deserialize) {
            (true, true) => quote!(#serde_root::Serialize, #serde_root::Deserialize),
            (true, false) => quote!(#serde_root::Serialize),
            (false, true) => quote!(#serde_root::Deserialize),
            (false, false) => return Ok(()),
        };

        // If we already have a derive, we want to insert ourselves into it
        if let Some(attr) = maybe_attr {
            let mut args =
                attr.parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?;
            args.push(parse_quote!(#new_attr));
            attr.tokens = quote!((#args));

        // Otherwise, we need to create the derive from scratch
        } else {
            let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[derive(#new_attr)]);
            let derive_attr = tmp_attrs.attributes.pop().unwrap();
            attrs.push(derive_attr);
        }

        // Detect typetag and include it within ent(...) if missing
        if !info.has_ent_typetag_attr {
            let maybe_attr = attrs.iter_mut().find(|a| {
                a.path
                    .segments
                    .last()
                    .filter(|s| s.ident == "ent")
                    .is_some()
            });

            let new_attr = quote!(typetag);

            // If we already have an ent(...), we want to insert ourselves into it
            if let Some(attr) = maybe_attr {
                let mut args =
                    attr.parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?;
                args.push(parse_quote!(#new_attr));
                attr.tokens = quote!((#args));

            // Otherwise, we need to create the ent(...) from scratch
            } else {
                let mut tmp_attrs: ParsableOuterAttributes = parse_quote!(#[ent(#new_attr)]);
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
) -> Result<(), syn::Error> {
    match (
        struct_info.has_conflicting_id_name,
        struct_info.has_id_marker,
    ) {
        (Some(span), None) => Err(syn::Error::new(span, "Conflicting field with same name")),
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
/// `database: Option<Box<dyn entity::Database>>` that has `#[ent(database)]`
/// included. This is only done if another field is not already marked as the
/// database. Will fail if a field already exists with the database's name and
/// there is no marked database field.
///
/// Additionally, the field will include `#[serde(skip)]` if it is detected
/// that `serde::Serialize` or `serde::Deserialize` has been derived or if
/// simple_ent(serde) has been specified.
///
/// The name can be altered via simple_ent(database = "...")
fn inject_ent_database_field(
    root: &Path,
    item: &mut ItemStruct,
    attr_info: &AttrInfo,
    struct_info: &StructInfo,
) -> Result<(), syn::Error> {
    match (
        struct_info.has_conflicting_database_name,
        struct_info.has_database_marker,
    ) {
        (Some(span), None) => Err(syn::Error::new(span, "Conflicting field with same name")),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_database_field_name);

                    // If we are deriving serde, we need to mark the database
                    // as skippable as it cannot be serialized. This can
                    // be forced to be included via simple_ent(serde) or
                    // forced excluded via simple_ent(no_serde)
                    let skip_attr = if (attr_info.is_deriving_serialize
                        || attr_info.is_deriving_deserialize
                        || attr_info.args_serde)
                        && !attr_info.args_no_serde
                    {
                        quote! { #[serde(skip)] }
                    } else {
                        quote! {}
                    };

                    let named_field: ParsableNamedField = parse_quote! {
                        #skip_attr
                        #[ent(database)]
                        #name: ::std::option::Option<::std::boxed::Box<dyn #root::Database>>
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
) -> Result<(), syn::Error> {
    match (
        struct_info.has_conflicting_created_name,
        struct_info.has_created_marker,
    ) {
        (Some(span), None) => Err(syn::Error::new(span, "Conflicting field with same name")),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_created_field_name);
                    let named_field: ParsableNamedField = parse_quote! {
                        #[ent(created)]
                        #name: u64
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
) -> Result<(), syn::Error> {
    match (
        struct_info.has_conflicting_last_updated_name,
        struct_info.has_last_updated_marker,
    ) {
        (Some(span), None) => Err(syn::Error::new(span, "Conflicting field with same name")),
        (_, Some(_)) => Ok(()),
        (None, None) => {
            match &mut item.fields {
                Fields::Named(x) => {
                    let name = format_ident!("{}", attr_info.target_last_updated_field_name);
                    let named_field: ParsableNamedField = parse_quote! {
                        #[ent(last_updated)]
                        #name: u64
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
    has_ent_typetag_attr: bool,
    args_no_serde: bool,
    args_serde: bool,
    args_no_derive_ent: bool,
    args_derive_ent: bool,
    args_no_derive_clone: bool,
    args_derive_clone: bool,
    target_id_field_name: String,
    target_database_field_name: String,
    target_created_field_name: String,
    target_last_updated_field_name: String,
}

impl AttrInfo {
    #[allow(clippy::field_reassign_with_default)]
    pub fn from(args: &[NestedMeta], attrs: &[Attribute]) -> Self {
        let mut args_attrs = utils::nested_meta_iter_into_named_attr_map(args.iter());
        let ent_attrs = utils::attrs_into_attr_map(attrs, "ent").unwrap_or_default();
        let derive_attrs = utils::attrs_into_attr_map(attrs, "derive").unwrap_or_default();

        let mut info = AttrInfo::default();

        info.is_deriving_clone = derive_attrs.get("Clone").copied().unwrap_or_default();
        info.is_deriving_ent = derive_attrs.get("Ent").copied().unwrap_or_default();
        info.is_deriving_serialize = derive_attrs.get("Serialize").copied().unwrap_or_default();
        info.is_deriving_deserialize = derive_attrs.get("Deserialize").copied().unwrap_or_default();
        info.has_ent_typetag_attr = ent_attrs.get("typetag").copied().unwrap_or_default();

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
        info.args_serde = args_attrs.contains_key("serde");
        info.args_no_derive_ent = args_attrs.contains_key("no_derive_ent");
        info.args_derive_ent = args_attrs.contains_key("derive_ent");
        info.args_no_derive_clone = args_attrs.contains_key("no_derive_clone");
        info.args_derive_clone = args_attrs.contains_key("derive_clone");

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
    pub fn from(args: &[NestedMeta], input: &ItemStruct) -> Result<Self, syn::Error> {
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
                return Err(syn::Error::new(input.span(), "Tuple struct not supported"))
            }
            Fields::Unit => return Err(syn::Error::new(input.span(), "Unit struct not supported")),
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
