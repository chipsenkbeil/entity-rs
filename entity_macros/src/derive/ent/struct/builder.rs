use crate::data::r#struct::Ent;
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Path, Type};

pub fn do_derive_ent_builder(root: Path, ent: Ent) -> darling::Result<TokenStream> {
    let ent_name = &ent.ident;
    let builder_name = format_ident!("{}Builder", ent_name);
    let builder_error_name = format_ident!("{}Error", builder_name);

    let vis = &ent.vis;
    let ent_database_field_name = &ent.database;

    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    let mut struct_field_names = Vec::new();
    let mut struct_field_defaults = Vec::new();
    let mut struct_fields = Vec::new();
    let mut struct_setters = Vec::new();
    let mut error_variants = Vec::new();
    let mut error_variant_field_names = Vec::new();
    let mut build_assignments = Vec::new();
    let mut has_normal_struct_field = false;

    push_id_field(
        &root,
        &ent.id,
        &ent.id_ty,
        &mut struct_field_names,
        &mut struct_fields,
        &mut struct_field_defaults,
        &mut build_assignments,
        &mut struct_setters,
    );
    push_database_field(
        &root,
        &ent.database,
        &ent.database_ty,
        &mut struct_field_names,
        &mut struct_fields,
        &mut struct_field_defaults,
        &mut build_assignments,
        &mut struct_setters,
    );
    push_timestamp_field(
        &root,
        &ent.created,
        &ent.created_ty,
        &mut struct_field_names,
        &mut struct_fields,
        &mut struct_field_defaults,
        &mut build_assignments,
        &mut struct_setters,
    );
    push_timestamp_field(
        &root,
        &ent.last_updated,
        &ent.last_updated_ty,
        &mut struct_field_names,
        &mut struct_fields,
        &mut struct_field_defaults,
        &mut build_assignments,
        &mut struct_setters,
    );

    for (name, ty) in ent
        .fields
        .iter()
        .map(|f| (&f.name, &f.ty))
        .chain(ent.edges.iter().map(|e| (&e.name, &e.ty)))
    {
        has_normal_struct_field = true;
        struct_field_names.push(name);
        struct_fields.push(quote!(#name: ::std::option::Option<#ty>));
        struct_field_defaults.push(quote!(::std::option::Option::None));

        let error_variant = format_ident!("Missing{}", name.to_string().to_camel_case());
        build_assignments.push(quote! {
            #name: self.#name.ok_or(#builder_error_name::#error_variant)?
        });
        error_variants.push(error_variant);
        error_variant_field_names.push(name);

        struct_setters.push(quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = ::std::option::Option::Some(value);
                self
            }
        });
    }

    let display_fmt_inner = if has_normal_struct_field {
        quote! {
            match self {
                #(
                    Self::#error_variants => ::std::write!(
                        f,
                        concat!("Missing ", ::std::stringify!(#error_variant_field_names)),
                    ),
                )*
            }
        }
    } else {
        quote!(::std::result::Result::Ok(()))
    };

    Ok(quote! {
        #[derive(
            ::std::marker::Copy,
            ::std::clone::Clone,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            ::std::cmp::Eq,
        )]
        #[automatically_derived]
        #[allow(clippy::enum_variant_names)]
        #vis enum #builder_error_name {
            #(#error_variants),*
        }

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #display_fmt_inner
            }
        }

        impl ::std::error::Error for #builder_error_name {}

        impl #impl_generics #ent_name #ty_generics #where_clause {
            /// Begin building a new ent, initialized using the global database
            /// if it is available
            pub fn build() -> #builder_name #ty_generics #where_clause {
                <#builder_name #ty_generics as ::std::default::Default>::default()
                    .#ent_database_field_name(#root::global::db())
            }
        }

        #[automatically_derived]
        #vis struct #builder_name #ty_generics #where_clause {
            #(#struct_fields),*
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #builder_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(
                        #struct_field_names: #struct_field_defaults,
                    )*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#struct_setters)*
        }

        #[automatically_derived]
        impl #impl_generics #root::EntBuilder for #builder_name #ty_generics #where_clause {
            type Output = #ent_name #ty_generics;
            type Error = #builder_error_name;

            /// Called when finished constructing the ent, will consume the
            /// builder and return a new ent **without** committing it to
            /// the database.
            fn finish(self) -> ::std::result::Result<Self::Output, Self::Error> {
                ::std::result::Result::Ok(#ent_name {
                    #(#build_assignments),*
                })
            }

            /// Called when finished constructing the ent, will consume the
            /// builder and return a new ent after committing it to the
            /// associated database. If no database is connected to the ent,
            /// this will fail.
            fn finish_and_commit(self) -> ::std::result::Result<
                #root::DatabaseResult<Self::Output>,
                Self::Error,
            > {
                self.finish().map(|mut ent| {
                    if let ::std::result::Result::Err(x) = #root::Ent::commit(&mut ent) {
                        ::std::result::Result::Err(x)
                    } else {
                        ::std::result::Result::Ok(ent)
                    }
                })
            }
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn push_id_field<'a, 'b>(
    root: &'a Path,
    name: &'b Ident,
    ty: &'b Type,
    names: &mut Vec<&'b Ident>,
    defs: &mut Vec<TokenStream>,
    defaults: &mut Vec<TokenStream>,
    assignments: &mut Vec<TokenStream>,
    setters: &mut Vec<TokenStream>,
) {
    names.push(name);
    defs.push(quote!(#name: #ty));
    defaults.push(quote!(#root::EPHEMERAL_ID));
    assignments.push(quote!(#name: self.#name));
    setters.push(quote! {
        pub fn #name(mut self, value: #ty) -> Self {
            self.#name = value;
            self
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn push_database_field<'a, 'b>(
    root: &'a Path,
    name: &'b Ident,
    ty: &'b Type,
    names: &mut Vec<&'b Ident>,
    defs: &mut Vec<TokenStream>,
    defaults: &mut Vec<TokenStream>,
    assignments: &mut Vec<TokenStream>,
    setters: &mut Vec<TokenStream>,
) {
    names.push(name);
    defs.push(quote!(#name: #ty));
    defaults.push(quote!(#root::WeakDatabaseRc::new()));
    assignments.push(quote!(#name: self.#name));
    setters.push(quote! {
        pub fn #name(mut self, value: #ty) -> Self {
            self.#name = value;
            self
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn push_timestamp_field<'a, 'b>(
    _root: &'a Path,
    name: &'b Ident,
    ty: &'b Type,
    names: &mut Vec<&'b Ident>,
    defs: &mut Vec<TokenStream>,
    defaults: &mut Vec<TokenStream>,
    assignments: &mut Vec<TokenStream>,
    setters: &mut Vec<TokenStream>,
) {
    names.push(name);
    defs.push(quote!(#name: #ty));
    defaults.push(quote!(::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .expect("Corrupt system time")
        .as_millis() as ::std::primitive::u64));
    assignments.push(quote!(#name: self.#name));
    setters.push(quote! {
        pub fn #name(mut self, value: #ty) -> Self {
            self.#name = value;
            self
        }
    });
}
