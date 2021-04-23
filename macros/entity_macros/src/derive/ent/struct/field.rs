use entity_macros_data::StructEnt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

pub fn do_derive_ent_typed_fields(_root: Path, ent: StructEnt) -> TokenStream {
    let name = &ent.ident;
    let mut field_methods: Vec<TokenStream> = Vec::new();
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    for field in ent.fields {
        let field_name = &field.name;
        let field_type = &field.ty;

        // If the field is computed, we treat it as such
        if let Some(computed) = field.computed {
            let expr = &computed.expr;
            let return_ty = &computed.return_ty;

            // We support a cached version of the getter, which will modify the
            // Option<...> based on cache status -- this requires the method
            // to take a mutable reference, which is why we distinguish it
            let method_name = format_ident!("get_or_compute_{}", field_name);
            field_methods.push(quote! {
                pub fn #method_name(&mut self) -> #return_ty {
                    let field_ref = ::std::option::Option::as_ref(&self.#field_name);
                    match field_ref {
                        ::std::option::Option::Some(x) =>
                            ::std::clone::Clone::clone(x),
                        ::std::option::Option::None => {
                            let res = #expr;
                            self.#field_name = ::std::option::Option::Some(
                                ::std::clone::Clone::clone(&res),
                            );
                            res
                        }
                    }
                }
            });

            // We also provide the standard getter, which will always re-compute
            // the value and not cache it
            field_methods.push(quote! {
                pub fn #field_name(&self) -> #return_ty {
                    #expr
                }
            });

        // Otherwise, the field is a normal piece of data found within the ent
        } else {
            let getter = quote! {
                pub fn #field_name(&self) -> &#field_type {
                    &self.#field_name
                }
            };
            field_methods.push(getter);

            if field.mutable {
                let setter_name = format_ident!("set_{}", field_name);
                let setter = quote! {
                    pub fn #setter_name(&mut self, x: #field_type) -> #field_type {
                        ::std::mem::replace(&mut self.#field_name, x)
                    }
                };
                field_methods.push(setter);
            }
        }
    }

    quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#field_methods)*
        }
    }
}
