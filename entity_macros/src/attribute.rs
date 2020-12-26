use crate::utils;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Field, Fields, ItemStruct, Token,
};

pub fn do_include_ent_core(
    root: TokenStream,
    mut input: ItemStruct,
) -> Result<TokenStream, syn::Error> {
    // Ensure that we have no conflicting field
    let new_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        #[ent(id)]
        id: #root::Id,

        #[ent(database)]
        database: ::std::option::Option<::std::boxed::Box<dyn #root::Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    };

    match &mut input.fields {
        Fields::Named(x) => {
            // Ensure we have no conflicts
            for f in x.named.iter() {
                let name = f.ident.as_ref().unwrap().to_string();
                match name.as_str() {
                    "id" | "database" | "created" | "last_updated" => {
                        return Err(syn::Error::new(
                            f.span(),
                            format!("Field named {} reserved for ent", name),
                        ))
                    }
                    _ => {}
                }

                if utils::has_outer_ent_attr(&f.attrs, "id") {
                    return Err(syn::Error::new(
                        f.span(),
                        "Cannot mark an id field if including ent core",
                    ));
                } else if utils::has_outer_ent_attr(&f.attrs, "database") {
                    return Err(syn::Error::new(
                        f.span(),
                        "Cannot mark a database field if including ent core",
                    ));
                } else if utils::has_outer_ent_attr(&f.attrs, "created") {
                    return Err(syn::Error::new(
                        f.span(),
                        "Cannot mark a created field if including ent core",
                    ));
                } else if utils::has_outer_ent_attr(&f.attrs, "last_updated") {
                    return Err(syn::Error::new(
                        f.span(),
                        "Cannot mark a last_updated field if including ent core",
                    ));
                }
            }

            x.named.extend(new_fields.into_iter().map(|p| p.field))
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new(input.span(), "Tuple struct not supported"))
        }
        Fields::Unit => return Err(syn::Error::new(input.span(), "Unit struct not supported")),
    }

    Ok(quote! { #input })
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

// match input.fields {
//     Fields::Named(mut x) => {
//         let f: Field = parse_quote! {
//             #[ent(id)]
//             id: #root::Id
//         };
//         x.named.push(f);
//     }
//     _ => {}
// }
