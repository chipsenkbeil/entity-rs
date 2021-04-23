use entity_macros_data::StructEnt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn do_derive_ent_debug(_root: Path, ent: StructEnt) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();
    let core_ent_fields = [ent.id, ent.created, ent.last_updated];
    let debug_fields = core_ent_fields
        .iter()
        .chain(ent.fields.iter().map(|f| &f.name))
        .chain(ent.edges.iter().map(|e| &e.name));

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
