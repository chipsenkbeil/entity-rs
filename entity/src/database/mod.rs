use crate::{
    ent::{Ent, Query},
    Id,
};
use derive_more::Display;

mod kvstore;
pub use kvstore::*;

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

    #[display(fmt = "Missing Ent: {}", id)]
    MissingEnt { id: Id },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: crate::ent::ValueType,
        actual: crate::ent::ValueType,
    },

    #[display(fmt = "Corrupted Ent {}: {}", id, source)]
    CorruptedEnt {
        id: Id,
        source: Box<dyn std::error::Error>,
    },

    #[display(fmt = "Ent Capacity Reached")]
    EntCapacityReached,
}

impl std::error::Error for DatabaseError {}

/// Represents a synchronous database, which performs blocking CRUD
/// operations using ents. All operations only require a reference to the
/// database and it is up to each implementation to provide proper
/// locking and safeguards to ensure that multi-threaded access does
/// not cause problems.
///
/// The database is required to be cloneable, which should not be an expensive
/// operation as cloning should only duplicate the underlying connection to
/// the database, not clone the database itself.
pub trait Database: Clone {
    /// Retrieves a copy of a single, generic ent with the corresponding id
    fn get(&self, id: Id) -> DatabaseResult<Option<Ent>>;

    /// Removes the ent with the corresponding id, triggering edge
    /// processing for all disconnected ents. Returns a boolean indicating
    /// if an ent was removed.
    fn remove(&self, id: Id) -> DatabaseResult<bool>;

    /// Inserts a new ent using its id as the primary index, overwriting
    /// any ent with a matching id. If the ent's id is set to the ephemeral
    /// id (of 0), a unique id will be assigned to the ent prior to being
    /// inserted.
    ///
    /// The ent's id is returned after being inserted.
    fn insert(&self, ent: impl Into<Ent>) -> DatabaseResult<Id>;
}

/// Represents extensions to the database to provide advanced functionality.
pub trait DatabaseExt {
    /// Performs a retrieval of multiple ents
    fn get_all(&self, ids: impl IntoIterator<Item = Id>) -> DatabaseResult<Vec<Ent>>;

    /// Finds all generic ents that match the query
    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Ent>>;
}
