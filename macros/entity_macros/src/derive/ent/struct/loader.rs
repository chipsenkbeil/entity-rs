use entity_macros_data::StructEnt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn do_derive_ent_loader(root: Path, ent: StructEnt) -> TokenStream {
    let name = &ent.ident;
    let (impl_generics, ty_generics, where_clause) = ent.generics.split_for_impl();

    quote! {
        impl #impl_generics #root::EntLoader for #name #ty_generics #where_clause {
            type Output = #name #ty_generics;

            fn load_from_db(
                db: #root::WeakDatabaseRc,
                id: #root::Id,
            ) -> #root::DatabaseResult<::std::option::Option<Self::Output>> {
                let database = #root::WeakDatabaseRc::upgrade(&db)
                    .ok_or(#root::DatabaseError::Disconnected)?;
                let maybe_ent = #root::Database::get(
                    ::std::convert::AsRef::<#root::Database>::as_ref(
                        ::std::convert::AsRef::<
                            ::std::boxed::Box<dyn #root::Database>
                        >::as_ref(&database),
                    ),
                    id,
                )?;

                let maybe_typed_ent = maybe_ent.and_then(|ent| ent.to_ent::<Self>());

                ::std::result::Result::Ok(maybe_typed_ent)
            }
        }
    }
}
