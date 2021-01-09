use crate::{Database, DatabaseResult, Ent, Id};
use derive_more::{Constructor, IntoIterator};
use std::fmt::Debug;

mod filter;
pub use filter::*;

mod predicate;
pub use predicate::*;

/// Represents a query interface for some ent
pub trait EntQuery {
    type Output;

    /// Executes the query against the provided database, returning the query's
    /// output as the result
    fn execute<D: Database>(self, database: &D) -> DatabaseResult<Self::Output>;
}

/// Represents a generic query to find ents within some database
#[derive(Constructor, IntoIterator, Clone, Debug, Default)]
pub struct Query(Vec<Filter>);

impl EntQuery for Query {
    type Output = Vec<Box<dyn Ent>>;

    #[inline]
    fn execute<D: Database>(self, database: &D) -> DatabaseResult<Self::Output> {
        database.find_all(self)
    }
}

impl Query {
    /// Consumes query, producing a new query with the additional filter
    /// added to the end of the filters to be applied
    pub fn chain(self, filter: Filter) -> Self {
        let mut filters = self.0;
        filters.push(filter);

        Query::new(filters)
    }

    pub fn where_id<P: Into<TypedPredicate<Id>>>(self, p: P) -> Self {
        self.chain(Filter::where_id(p))
    }

    pub fn where_type<P: Into<TypedPredicate<String>>>(self, p: P) -> Self {
        self.chain(Filter::where_type(p))
    }

    pub fn where_created<P: Into<TypedPredicate<u64>>>(self, p: P) -> Self {
        self.chain(Filter::where_created(p))
    }

    pub fn where_last_updated<P: Into<TypedPredicate<u64>>>(self, p: P) -> Self {
        self.chain(Filter::where_last_updated(p))
    }

    pub fn where_field<S: Into<String>, P: Into<Predicate>>(self, name: S, p: P) -> Self {
        self.chain(Filter::where_field(name, p))
    }

    pub fn where_edge<S: Into<String>, F: Into<Filter>>(self, name: S, filter: F) -> Self {
        self.chain(Filter::where_edge(name, filter))
    }

    pub fn where_into_edge<S: Into<String>>(self, name: S) -> Self {
        self.chain(Filter::where_into_edge(name))
    }
}
