mod inmemory;

use super::DatabaseResult;
use crate::ent::{Ent, Query};

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
