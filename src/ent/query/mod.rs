use crate::{DatabaseError, DatabaseResult, Ent, Id, WeakDatabaseRc};
use derive_more::{Constructor, IntoIterator};
use std::{fmt::Debug, iter::Extend};

mod filter;
pub use filter::*;

mod predicate;
pub use predicate::*;

/// Represents a query interface for some ent
pub trait EntQuery: Sized {
    type Output;

    /// Executes the query against the provided database, returning the query's
    /// output as the result
    fn execute_with_db(self, db: WeakDatabaseRc) -> DatabaseResult<Self::Output>;

    /// Executes the query against the global database, returning the query's
    /// output as the result
    fn execute(self) -> DatabaseResult<Self::Output> {
        Self::execute_with_db(self, crate::global::db())
    }
}

/// Represents a generic query to find ents within some database
#[derive(Constructor, IntoIterator, Clone, Debug, Default)]
pub struct Query(Vec<Filter>);

impl EntQuery for Query {
    type Output = Vec<Box<dyn Ent>>;

    #[inline]
    fn execute_with_db(self, db: WeakDatabaseRc) -> DatabaseResult<Self::Output> {
        let database = WeakDatabaseRc::upgrade(&db).ok_or(DatabaseError::Disconnected)?;
        database.find_all(self)
    }
}

impl Extend<Filter> for Query {
    /// Extends the query's filters with the contents of the iterator
    fn extend<T: IntoIterator<Item = Filter>>(&mut self, iter: T) {
        self.0.extend(iter)
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

    /// Updates the query by adding an additional filter to the end
    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.0.push(filter);
        self
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
