use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::path::Path;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Fields, Ident, ItemStruct, Type, Visibility};

const ENT_ATTR: &str = "ent";

/// Transforms pseudo-struct syntax into an ent representation
///
/// ```
/// use entity_macros::ent;
///
/// ent! {
///     pub struct PageEnt {
///         #[ent(field, indexed)]
///         title: String,
///
///         #[ent(field)]
///         url: String,
///
///         #[ent(edge)]
///         header: ContentEnt,
///
///         #[ent(edge)]
///         subheader: Option<ContentEnt>,
///
///         #[ent(edge)]
///         paragraphs: Vec<ContentEnt>,
///     }
/// }
/// ```
#[proc_macro]
pub fn ent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: ItemStruct = parse_macro_input!(input as ItemStruct);

    let root = quote! { ::entity };
    let expanded = impl_ent(root, input).unwrap_or_else(|x| x.to_compile_error());

    proc_macro::TokenStream::from(expanded)
}

#[inline]
fn impl_ent(root: TokenStream, item_struct: ItemStruct) -> Result<TokenStream, syn::Error> {
    let name = item_struct.ident;
    let vis = item_struct.vis;
    let fields = item_struct.fields;
    let builder_name = format_ident!("{}Builder", name);
    let const_type_name = format_ident!("{}_TYPE", name);

    let ent_trait_t = impl_ent_trait(&root, &name);
    let into_ent_t = impl_into_ent(&root, &name);
    let database_operations_t = impl_database_operations(&root, &name, &builder_name);
    let fields_t = impl_fields(&root, &name, &fields)?;
    let edges_t = impl_edges(&root, &name, &fields);
    let try_from_ent_t = impl_try_from_ent(&root, &name, &const_type_name, &fields);
    let builder_t = impl_builder(&root, &builder_name, &fields);

    // Build the output, possibly using quasi-quotation
    Ok(quote! {
        #vis const #const_type_name: &str = concat!(module_path!(), "::", stringify!(#name));
        #vis struct #name(#root::Ent);
        #ent_trait_t
        #into_ent_t
        #database_operations_t
        #fields_t
        #edges_t
        #try_from_ent_t
        #builder_t
    })
}

#[inline]
fn impl_into_ent(root: &TokenStream, name: &Ident) -> TokenStream {
    quote! {
        impl ::std::convert::Into<#root::Ent> for #name {
            fn into(self) -> #root::Ent {
                self.0
            }
        }
    }
}

#[inline]
fn impl_ent_trait(root: &TokenStream, name: &Ident) -> TokenStream {
    quote! {
        impl #root::IEnt for #name {
            fn id(&self) -> #root::Id {
                self.0.id()
            }

            fn r#type(&self) -> &str {
                self.0.r#type()
            }

            fn created(&self) -> u64 {
                self.0.created()
            }

            fn last_updated(&self) -> u64 {
                self.0.last_updated()
            }

            fn fields(&self) -> ::std::vec::Vec<&#root::Field> {
                self.0.fields()
            }

            fn field(&self, name: &str) -> ::std::option::Option<&#root::Field> {
                self.0.field(name)
            }

            fn edges(&self) -> ::std::vec::Vec<&#root::Edge> {
                self.0.edges()
            }

            fn edge(&self, name: &str) -> ::std::option::Option<&#root::Edge> {
                self.0.edge(name)
            }
        }
    }
}

#[inline]
fn impl_database_operations(root: &TokenStream, name: &Ident, builder_name: &Ident) -> TokenStream {
    quote! {
        /// Implementation for database-oriented operations
        impl #name {
            /// Refreshes ent by checking database for latest version and returning it
            pub fn refresh(&mut self) -> #root::DatabaseResult<()> {
                self.0.refresh()
            }

            /// Saves the ent to the database, updating this local instance's id
            /// if the database has reported a new id
            pub fn commit(&mut self) -> #root::DatabaseResult<()> {
                self.0.commit()
            }

            /// Removes self from database
            pub fn remove(self) -> #root::DatabaseResult<bool> {
                self.0.remove()
            }

            /// Retrieves ent from database with corresponding id and makes sure
            /// that it can be represented as a typed ent
            pub fn get_from_database<D: #root::Database>(
                db: D,
                id: #root::Id,
            ) -> #root::DatabaseResult<::std::option::Option<Self>> {
                use #root::IEnt;
                use ::std::convert::TryFrom;
                match db.get(id) {
                    ::std::result::Result::Ok(::std::option::Option::Some(ent)) => {
                        let id = ent.id();
                        let x =
                            #name::try_from(ent).map_err(|e| #root::DatabaseError::CorruptedEnt {
                                id,
                                source: ::std::boxed::Box::from(e),
                            })?;
                        ::std::result::Result::Ok(::std::option::Option::Some(Self(x.into())))
                    }
                    ::std::result::Result::Ok(::std::option::Option::None) => {
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }
                    ::std::result::Result::Err(x) => ::std::result::Result::Err(x),
                }
            }

            /// Produces a new ent builder for the given database
            pub fn build_with_database<D: #root::Database + 'static>(db: D) -> #builder_name {
                #builder_name::default().database(db)
            }
        }
    }
}

#[inline]
fn impl_fields(
    root: &TokenStream,
    name: &Ident,
    fields: &Fields,
) -> Result<TokenStream, syn::Error> {
    fn impl_field_accessor(
        _root: &TokenStream,
        vis: &Visibility,
        name: &Ident,
        ftype: &Type,
    ) -> TokenStream {
        quote! {
            #vis fn #name(&self) -> &#ftype {
                use ::std::convert::TryFrom;
                #ftype::try_from(
                    self.field(stringify!(#name))
                        .expect(concat!("Missing field: ", #name))
                ).expect("Unexpected field value")
            }
        }
    }

    fn impl_field_mutator(
        root: &TokenStream,
        vis: &Visibility,
        name: &Ident,
        _ftype: &Type,
    ) -> TokenStream {
        let setter_name = format_ident!("set_{}", name);
        quote! {
            #vis fn #setter_name<VALUE: ::std::convert::Into<::std::string::String>>(
                &mut self,
                value: VALUE,
            ) {
                self.0
                    .update_field(stringify!(#name), #root::Value::from(value.into()))
                    .expect(concat!("Corrupted ent field: ", stringify!(#name)));
            }
        }
    }

    let named_fields = match fields {
        Fields::Named(x) => x.named.iter(),
        Fields::Unnamed(x) => return Err(syn::Error::new(x.span(), "Named fields required")),
        Fields::Unit => return Err(syn::Error::new(name.span(), "Struct cannot be unit")),
    };

    // TODO: Support optionality of ent mutators (may want ent to have immutable by default)
    let (getters, setters): (Vec<TokenStream>, Vec<TokenStream>) = named_fields
        .filter(|f| f.attrs.iter().any(|a| a.path.is_ident(ENT_ATTR)))
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let vis = &f.vis;
            let r#type = &f.ty;
            (
                impl_field_accessor(root, vis, name, r#type),
                impl_field_mutator(root, vis, name, r#type),
            )
        })
        .fold((Vec::new(), Vec::new()), |(mut g, mut s), (g_t, s_t)| {
            g.push(g_t);
            s.push(s_t);
            (g, s)
        });

    Ok(quote! {
        impl #name {
            #(#getters)*
            #(#setters)*
        }
    })
}

#[inline]
fn impl_edges(root: &TokenStream, name: &Ident, fields: &Fields) -> TokenStream {
    quote! {
        impl #name {
        }
    }
}

#[inline]
fn impl_try_from_ent(
    root: &TokenStream,
    name: &Ident,
    const_type_name: &Ident,
    fields: &Fields,
) -> TokenStream {
    quote! {
        impl ::std::convert::TryFrom<#root::Ent> for $name {
            type Error = #root::EntConversionError;

            fn try_from(ent: #root::Ent) -> ::std::result::Result<Self, Self::Error> {
                use #root::IEnt;

                if ent.r#type() != #const_type_name {
                    return ::std::result::Result::Err(#root::EntConversionError::EntWrongType {
                        expected: #const_type_name.to_string(),
                        actual: ent.r#type().to_string(),
                    });
                }

                $(
                    match ent.field_value(stringify!($fname)) {
                        ::std::option::Option::None => {
                            return ::std::result::Result::Err($#root::EntConversionError::FieldMissing {
                                name: stringify!($fname).to_string(),
                            });
                        }
                        ::std::option::Option::Some(x)
                            if x.to_type()
                                != #root::ValueType::from_type_name(stringify!($ftype))
                                    .expect("Invalid field type") =>
                        {
                            return ::std::result::Result::Err(#root::EntConversionError::FieldWrongType {
                                name: stringify!($fname).to_string(),
                                expected: #root::ValueType::from_type_name(stringify!($ftype))
                                    .expect("Invalid field type"),
                                actual: x.to_type(),
                            });
                        }
                        _ => {}
                    }
                )*

                $(
                    ent!(@private @check_edge $kind $ename);
                )*

                ::std::result::Result::Ok(Self(ent))
            }
        }
    }
}

#[inline]
fn impl_builder(root: &TokenStream, builder_name: &Ident, fields: &Fields) -> TokenStream {
    let builder_error_name = format_ident!("{}Error", builder_name);
    quote! {
        #[derive(Debug)]
        pub enum #builder_error_name {
            MissingDatabase,
            $([<Missing $fname:camel Field>],)*
            $([<Missing $ename:camel Edge>],)*
        }

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::MissingDatabase => write!(f, "Missing database"),
                    $(
                        Self::[<Missing $fname:camel Field>] => write!(f, concat!("Missing ", stringify!([<$fname>]), " field")),
                    )*
                    $(
                        Self::[<Missing $ename:camel Edge>] => write!(f, concat!("Missing ", stringify!([<$ename>]), " edge")),
                    )*
                }
            }
        }

        impl ::std::error::Error for #builder_error_name {}

        pub struct #builder_name {
            database: ::std::option::Option<::std::boxed::Box<dyn #root::Database>>,
            $(
                [<$fname>]: ::std::option::Option<$ftype>,
            )*
            $(
                [<$ename>]: ::std::option::Option<ent!(@private @edge_type $kind #root::Id)>,
            )*
        }

        impl ::std::default::Default for #builder_name {
            fn default() -> Self {
                Self {
                    database: ::std::option::Option::default(),
                    $(
                        [<$fname>]: ::std::option::Option::default(),
                    )*
                    $(
                        [<$ename>]: ::std::option::Option::default(),
                    )*
                }
            }
        }

        impl #builder_name {
            pub fn database<VALUE: #root::Database + 'static>(mut self, value: VALUE) -> Self {
                self.database_boxed(::std::boxed::Box::from(value))
            }

            pub fn database_boxed(mut self, value: ::std::boxed::Box<dyn #root::Database>) -> Self {
                self.database = ::std::option::Option::Some(value);
                self
            }

            $(
                pub fn [<$fname:snake:lower>]<VALUE: ::std::convert::Into<$ftype>>(
                    mut self,
                    value: VALUE,
                ) -> Self {
                    self.[<$fname:snake:lower>] = ::std::option::Option::Some(value.into());
                    self
                }
            )*

            $(
                pub fn [<$ename:snake:lower>]<VALUE: ::std::convert::Into<ent!(@private @edge_type $kind #root::Id)>>(
                    mut self,
                    value: VALUE,
                ) -> Self {
                    self.[<$ename:snake:lower>] = ::std::option::Option::Some(value.into());
                    self
                }
            )*

            pub fn create(self) -> ::std::result::Result<$name, #builder_error_name> {
                let mut fields = ::std::vec::Vec::new();
                let mut edges = ::std::vec::Vec::new();

                $(
                    let mut field_attrs = ::std::vec::Vec::new();
                    $(ent!(@private @push_field_attr @indexed field_attrs);)?
                    fields.push(#root::Field::new_with_attributes(
                        stringify!([<$fname>]),
                        self.[<$fname>].ok_or([<$name BuilderError::Missing $fname:camel Field>])?,
                        field_attrs,
                    ));
                )*

                $(
                    edges.push(#root::Edge::new_with_deletion_policy(
                        stringify!([<$ename>]),
                        self.[<$ename>].ok_or([<$name BuilderError::Missing $ename:camel Edge>])?,
                        ent!(@private @edge_policy $policy),
                    ));
                )*

                let database = self.database.ok_or([<$name BuilderError::MissingDatabase>])?;
                let mut ent =
                    #root::Ent::from_collections(#root::EPHEMERAL_ID, [<$name _TYPE>], fields, edges);
                ent.connect_boxed(database);

                ::std::result::Result::Ok($name(ent))
            }
        }
    }
}
