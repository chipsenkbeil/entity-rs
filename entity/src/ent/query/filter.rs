use crate::{Id, Predicate, TypedPredicate};

/// Represents some filter to apply against an ent when searching through
/// a database
#[derive(Clone, Debug)]
pub enum Filter {
    /// Filters by the ent's id
    Id(TypedPredicate<Id>),

    /// Filters by the ent's type
    Type(TypedPredicate<String>),

    /// Filters by the ent's creation timestamp
    Created(TypedPredicate<u64>),

    /// Filters by the ent's last updated timestamp
    LastUpdated(TypedPredicate<u64>),

    /// Filters by an ent's field
    Field(String, Predicate),

    /// Filters by an ent connected by an edge; not the same as
    /// [`Filter::IntoEdge`], which converts an ent to its edge's ents
    Edge(String, Box<Filter>),

    /// **(Special case)** Filters by converting an ent into the ents on its edge
    IntoEdge(String),
}

impl Filter {
    pub fn where_id<P: Into<TypedPredicate<Id>>>(p: P) -> Self {
        Self::Id(p.into())
    }

    pub fn where_type<P: Into<TypedPredicate<String>>>(p: P) -> Self {
        Self::Type(p.into())
    }

    pub fn where_created<P: Into<TypedPredicate<u64>>>(p: P) -> Self {
        Self::Created(p.into())
    }

    pub fn where_last_updated<P: Into<TypedPredicate<u64>>>(p: P) -> Self {
        Self::LastUpdated(p.into())
    }

    pub fn where_field<S: Into<String>, P: Into<Predicate>>(name: S, p: P) -> Self {
        Self::Field(name.into(), p.into())
    }

    pub fn where_edge<S: Into<String>, F: Into<Filter>>(name: S, filter: F) -> Self {
        Self::Edge(name.into(), Box::new(filter.into()))
    }

    pub fn where_into_edge<S: Into<String>>(name: S) -> Self {
        Self::IntoEdge(name.into())
    }
}
