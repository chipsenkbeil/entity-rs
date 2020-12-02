/// Generates a new ent using the provided definition to outline its name,
/// fields, and edges for use in an application.
///
/// ## Examples
///
/// ```
/// use entity::{ent, InmemoryDatabase};
///
/// ent! {
///     @name PageEnt;
///     @fields(
///         title String;
///     );
///     @edges(
///         @one header ContentEnt;
///         @maybe subheader ContentEnt;
///         @many paragraphs ContentEnt;
///     );
/// }
///
/// ent! {
///     @name ContentEnt;
///     @fields(
///         text String;
///     );
/// }
///
/// let db = InmemoryDatabase::default();
/// let page = PageEnt::new_at(
///     db,
///     String::from("some title")),
///
///
/// ```
#[macro_export]
macro_rules! ent {
    (@private @edge_type @maybe $type:ty) => {
        ::std::option::Option<$type>
    };
    (@private @edge_type @one $type:ty) => {
        $type
    };
    (@private @edge_type @many $type:ty) => {
        ::std::vec::Vec<$type>
    };
    (@private @edge_policy @deep) => {
        $crate::EdgeDeletionPolicy::DeepDelete,
    };
    (@private @edge_policy @shallow) => {
        $crate::EdgeDeletionPolicy::ShallowDelete,
    };
    (@private @edge_policy @nothing) => {
        $crate::EdgeDeletionPolicy::Nothing,
    };
    (@private @edge_policy) => {
        $crate::EdgeDeletionPolicy::ShallowDelete,
    };
    (@private @push_field_attr @indexed $vec:ident) => {
        $vec.push(crate::FieldAttribute::Indexed);
    };
    (@private @check_edge @maybe $name:ident) => {
        match ent.edge(stringify!($name)) {
            ::std::option::Option::None => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeMissing {
                    name: stringify!($name).to_string(),
                });
            }
            ::std::option::Option::Some(x) if x.to_type() != $crate::EdgeValueType::MaybeOne => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeWrongType {
                    name: stringify!($name).to_string(),
                    expected: $crate::EdgeValueType::MaybeOne,
                    actual: x.to_type(),
                });
            }
            _ => {}
        }
    };
    (@private @check_edge @one $name:ident) => {
        match ent.edge(stringify!($name)) {
            ::std::option::Option::None => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeMissing {
                    name: stringify!($name).to_string(),
                });
            }
            ::std::option::Option::Some(x) if x.to_type() != $crate::EdgeValueType::One => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeWrongType {
                    name: stringify!($name).to_string(),
                    expected: $crate::EdgeValueType::One,
                    actual: x.to_type(),
                });
            }
            _ => {}
        }
    };
    (@private @check_edge @many $name:ident) => {
        match ent.edge(stringify!($name)) {
            ::std::option::Option::None => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeMissing {
                    name: stringify!($name).to_string(),
                });
            }
            ::std::option::Option::Some(x) if x.to_type() != $crate::EdgeValueType::Many => {
                return ::std::result::Result::Err($crate::EntConversionError::EdgeWrongType {
                    name: stringify!($name).to_string(),
                    expected: $crate::EdgeValueType::Many,
                    actual: x.to_type(),
                });
            }
            _ => {}
        }
    };
    (@private @define_load_edge @maybe $name:ident $type:ty) => {
        pub fn [<load_ $name:snake:lower>](&self) -> $crate::DatabaseResult<::std::option::Option<$type>> {
            use ::std::convert::TryFrom;
            self.0
                .load_edge(stringify!($name))?
                .into_iter()
                .nth(0)
                .map(|ent| {
                    use $crate::IEnt;
                    let id = ent.id();
                    $type::try_from(ent).map_err(|e| $crate::DatabaseError::CorruptedEnt {
                        id,
                        source: ::std::boxed::Box::from(e),
                    })
                })
                .transpose()
        }
    };
    (@private @define_load_edge @one $name:ident $type:ty) => {
        pub fn [<load_ $name:snake:lower>](&self) -> $crate::DatabaseResult<$type> {
            use ::std::convert::TryFrom;
            let ent = self
                .0
                .load_edge(stringify!($name))?
                .into_iter()
                .nth(0)
                .ok_or($crate::DatabaseError::BrokenEdge {
                    name: stringify!($name).to_string(),
                })?;

            use $crate::IEnt;
            let id = ent.id();
            $type::try_from(ent).map_err(|e| $crate::DatabaseError::CorruptedEnt {
                id,
                source: ::std::boxed::Box::from(e),
            })
        }
    };
    (@private @define_load_edge @many $name:ident $type:ty) => {
        pub fn [<load_ $name:snake:lower>](&self) -> $crate::DatabaseResult<::std::vec::Vec<$type>> {
            use ::std::convert::TryFrom;
            self.0
                .load_edge(stringify!($name))?
                .into_iter()
                .map(|ent| {
                    use $crate::IEnt;
                    let id = ent.id();
                    $type::try_from(ent).map_err(|e| $crate::DatabaseError::CorruptedEnt {
                        id,
                        source: ::std::boxed::Box::from(e),
                    })
                })
                .collect()
        }
    };

    (
        @name $name:ident;
        $(@attrs($(@attr $aname:ident;)*);)?
        $(@fields($($(@indexed)? $fname:ident $ftype:ty;)*);)?
        $(@edges($($kind:tt $($policy:tt)? $ename:ident $etype:ty;)*);)?
    ) => {
        paste! {
            pub const [<$name _TYPE>]: &str = concat!(module_path!(), "::", stringify!($name));

            pub struct $name($crate::Ent);

            impl ::std::convert::Into<$crate::Ent> for $name {
                fn into(self) -> $crate::Ent {
                    self.0
                }
            }

            /// Implementation for database-oriented operations
            impl $name {
                /// Refreshes ent by checking database for latest version and returning it
                pub fn refresh(&mut self) -> $crate::DatabaseResult<()> {
                    self.0.refresh()
                }

                /// Saves the ent to the database, updating this local instance's id
                /// if the database has reported a new id
                pub fn commit(&mut self) -> $crate::DatabaseResult<()> {
                    self.0.commit()
                }

                /// Removes self from database
                pub fn remove(self) -> $crate::DatabaseResult<bool> {
                    self.0.remove()
                }

                /// Retrieves ent from database with corresponding id and makes sure
                /// that it can be represented as a typed ent
                pub fn get_from_database<D: $crate::Database>(
                    db: D,
                    id: crate::Id,
                ) -> $crate::DatabaseResult<::std::option::Option<Self>> {
                    use $crate::IEnt;
                    use ::std::convert::TryFrom;
                    match db.get(id) {
                        ::std::result::Result::Ok(::std::option::Option::Some(ent)) => {
                            let id = ent.id();
                            let x =
                                PageEnt::try_from(ent).map_err(|e| $crate::DatabaseError::CorruptedEnt {
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
                pub fn build_with_database<D: $crate::Database + 'static>(db: D) -> [<$name Builder>] {
                    [<$name Builder>]::default().database(db)
                }
            }

            impl $name {
                $(
                    pub fn [<$ename:snake:lower>](&self) -> &$etype {
                        todo!("Need a Value -> specific type try_from conversion")
                    }
                )*

                // pub fn set_title<VALUE: ::std::convert::Into<::std::string::String>>(
                //     &mut self,
                //     value: VALUE,
                // ) {
                //     self.0
                //         .update_field(stringify!(title), crate::Value::from(value.into()))
                //         .expect(format!("Corrupted ent field: {}", stringify!(title)));
                // }
            }

            impl $name {
                $(
                    ent!(@private @define_load_edge $kind $ename $etype);
                )*
            }

            /// Implementation of IEnt interface
            impl $crate::IEnt for $name {
                fn id(&self) -> $crate::Id {
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

                fn fields(&self) -> ::std::vec::Vec<&$crate::Field> {
                    self.0.fields()
                }

                fn field(&self, name: &str) -> ::std::option::Option<&$crate::Field> {
                    self.0.field(name)
                }

                fn edges(&self) -> ::std::vec::Vec<&$crate::Edge> {
                    self.0.edges()
                }

                fn edge(&self, name: &str) -> ::std::option::Option<&$crate::Edge> {
                    self.0.edge(name)
                }
            }

            impl ::std::convert::TryFrom<$crate::Ent> for $name {
                type Error = $crate::EntConversionError;

                fn try_from(ent: $crate::Ent) -> ::std::result::Result<Self, Self::Error> {
                    use $crate::IEnt;

                    if ent.r#type() != [<$name _TYPE>] {
                        return ::std::result::Result::Err($crate::EntConversionError::EntWrongType {
                            expected: [<$name _TYPE>].to_string(),
                            actual: ent.r#type().to_string(),
                        });
                    }

                    $(
                        match ent.field_value(stringify!($fname)) {
                            ::std::option::Option::None => {
                                return ::std::result::Result::Err($crate::EntConversionError::FieldMissing {
                                    name: stringify!($fname).to_string(),
                                });
                            }
                            ::std::option::Option::Some(x)
                                if x.to_type()
                                    != $crate::ValueType::from_type_name(stringify!($ftype))
                                        .expect("Invalid field type") =>
                            {
                                return ::std::result::Result::Err($crate::EntConversionError::FieldWrongType {
                                    name: stringify!($fname).to_string(),
                                    expected: $crate::ValueType::from_type_name(stringify!($ftype))
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

            #[derive(Debug)]
            pub enum [<$name BuilderError>] {
                MissingDatabase,
                $([<Missing $fname:camel Field>],)*
                $([<Missing $ename:camel Edge>],)*
            }

            impl ::std::fmt::Display for [<$name BuilderError>] {
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

            impl ::std::error::Error for [<$name BuilderError>] {}

            pub struct [<$name Builder>] {
                database: ::std::option::Option<::std::boxed::Box<dyn $crate::Database>>,
                $(
                    [<$fname>]: ::std::option::Option<$ftype>,
                )*
                $(
                    [<$ename>]: ::std::option::Option<ent!(@private @edge_type $kind $crate::Id)>,
                )*
            }

            impl ::std::default::Default for [<$name Builder>] {
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

            impl [<$name Builder>] {
                pub fn database<VALUE: $crate::Database + 'static>(mut self, value: VALUE) -> Self {
                    self.database_boxed(::std::boxed::Box::from(value))
                }

                pub fn database_boxed(mut self, value: ::std::boxed::Box<dyn $crate::Database>) -> Self {
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
                    pub fn [<$ename:snake:lower>]<VALUE: ::std::convert::Into<ent!(@private @edge_type $kind $crate::Id)>>(
                        mut self,
                        value: VALUE,
                    ) -> Self {
                        self.[<$ename:snake:lower>] = ::std::option::Option::Some(value.into());
                        self
                    }
                )*

                pub fn create(self) -> ::std::result::Result<$name, [<$name BuilderError>]> {
                    let mut fields = ::std::vec::Vec::new();
                    let mut edges = ::std::vec::Vec::new();

                    $(
                        let mut field_attrs = ::std::vec::Vec::new();
                        $(ent!(@private @push_field_attr @indexed field_attrs);)?
                        fields.push($crate::Field::new_with_attributes(
                            stringify!([<$fname>]),
                            self.[<$fname>].ok_or([<$name BuilderError::Missing $fname:camel Field>])?,
                            field_attrs,
                        ));
                    )*

                    $(
                        edges.push($crate::Edge::new_with_deletion_policy(
                            stringify!([<$ename>]),
                            self.[<$ename>].ok_or([<$name BuilderError::Missing $ename:camel Edge>])?,
                            ent!(@private @edge_policy $policy),
                        ));
                    )*

                    let database = self.database.ok_or([<$name BuilderError::MissingDatabase>])?;
                    let mut ent =
                        $crate::Ent::from_collections($crate::EPHEMERAL_ID, [<$name _TYPE>], fields, edges);
                    ent.connect_boxed(database);

                    ::std::result::Result::Ok($name(ent))
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // @name $name:ident;
    // $(@attrs($(@attr $aname:ident)*);)?
    // $(@fields($($($fattr:tt)? $fname:ident $ftype:ty)*);)?
    // $(@edges($($kind:tt $($policy:tt)? $ename:ident $etype:ty)*);)?
    ent! {
        @name PageEnt;
        @fields(
            title String;
        );
    }
}
