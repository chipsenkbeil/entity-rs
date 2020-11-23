use crate::ent::{Ent, Query};
use derive_more::Display;

mod inmemory;
pub use inmemory::*;

/// Alias to a result that can contain a database error
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents some error the can occur when accessing the database
#[derive(Debug, Display)]
pub enum DatabaseError {
    #[display(fmt = "Connection Issue: {}", source)]
    Connection { source: Box<dyn std::error::Error> },

    #[display(fmt = "Missing Field: {}", name)]
    MissingField { name: String },

    #[display(fmt = "Missing Edge: {}", name)]
    MissingEdge { name: String },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: crate::ent::ValueType,
        actual: crate::ent::ValueType,
    },
}

impl std::error::Error for DatabaseError {}

/// Represents a synchronous database, which performs blocking CRUD
/// operations using ents. All operations only require a reference to the
/// database and it is up to each implementation to provide proper
/// locking and safeguards to ensure that multi-threaded access does
/// not cause problems.
pub trait Database {
    /// Retrieves a copy of a single, generic ent with the corresponding id
    fn get(&self, id: usize) -> DatabaseResult<Option<Ent>>;

    /// Removes the ent with the corresponding id, triggering edge
    /// processing for all disconnected ents.
    fn remove(&self, id: usize) -> DatabaseResult<()>;

    /// Inserts a new ent using its id as the primary index, overwriting
    /// any ent with a matching id.
    fn insert(&self, ent: impl Into<Ent>) -> DatabaseResult<()>;
}

/// Represents extensions to the database to provide advanced functionality.
pub trait DatabaseExt {
    /// Performs a retrieval of multiple ents
    fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> DatabaseResult<Vec<Ent>>;

    /// Finds all generic ents that match the query
    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Ent>>;
}
