use super::utils;
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn impl_ent_builder(
    _root: &TokenStream,
    input: &DeriveInput,
) -> Result<TokenStream, syn::Error> {
    let ent_name = &input.ident;
    let builder_name = format_ident!("{}Builder", ent_name);
    let builder_error_name = format_ident!("{}Error", builder_name);

    let vis = &input.vis;
    let named_fields = &utils::get_named_fields(input)?.named;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut struct_field_names = Vec::new();
    let mut struct_fields = Vec::new();
    let mut struct_setters = Vec::new();
    let mut error_variants = Vec::new();

    for f in named_fields {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        struct_field_names.push(name);
        struct_fields.push(quote!(#name: ::std::option::Option<#ty>));
        struct_setters.push(quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = ::std::option::Option::Some(value);
                self
            }
        });
        error_variants.push(format_ident!("Missing{}", name.to_string().to_camel_case()));
    }

    Ok(quote! {
        #[derive(::std::fmt::Debug)]
        #vis enum #builder_error_name {
            #(#error_variants),*
        }

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(
                        Self::#error_variants => write!(
                            f,
                            concat!("Missing ", stringify!(#struct_field_names)),
                        ),
                    )*
                }
            }
        }

        impl ::std::error::Error for #builder_error_name {}

        #[derive(::std::default::Default)]
        #vis struct #builder_name #ty_generics #where_clause {
            #(#struct_fields),*
        }

        #[automatically_derived]
        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#struct_setters)*

            pub fn build(self) -> ::std::result::Result<#ent_name #ty_generics, #builder_error_name> {
                ::std::result::Result::Ok(#ent_name {
                    #(
                        #struct_field_names: self.#struct_field_names.ok_or(
                            #builder_error_name::#error_variants
                        )?,
                    )*
                })
            }
        }
    })
}
