mod kv;
pub use kv::*;

use crate::{
    ent::{Ent, Query, ValueType},
    Id,
};
use derive_more::Display;
use std::sync::{Arc, Weak};

/// Represents a thread-safe reference to a boxed database trait object
pub type DatabaseRc = Arc<Box<dyn Database>>;

/// Represents a weak thread-safe reference to a boxed database trait object
pub type WeakDatabaseRc = Weak<Box<dyn Database>>;

/// Alias to a result that can contain a database error
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents some error the can occur when accessing the database
#[derive(Debug, Display)]
pub enum DatabaseError {
    #[display(fmt = "Connection Issue: {}", source)]
    Connection { source: Box<dyn std::error::Error> },

    #[display(fmt = "Disconnected")]
    Disconnected,

    #[display(fmt = "Missing Field: {}", name)]
    MissingField { name: String },

    #[display(fmt = "Missing Edge: {}", name)]
    MissingEdge { name: String },

    #[display(fmt = "Missing Ent: {}", id)]
    MissingEnt { id: Id },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: ValueType,
        actual: ValueType,
    },

    #[display(fmt = "Corrupted Ent {}: {}", id, source)]
    CorruptedEnt {
        id: Id,
        source: Box<dyn std::error::Error>,
    },

    #[display(fmt = "Broken Edge {}", name)]
    BrokenEdge { name: String },

    #[display(fmt = "Ent Capacity Reached")]
    EntCapacityReached,

    #[display(fmt = "{}", source)]
    Other { source: Box<dyn std::error::Error> },
}

impl std::error::Error for DatabaseError {}

/// Represents a synchronous database, which performs blocking CRUD
/// operations using ents. Given that many database implementations handle
/// interior mutability themselves, the API of this trait does not provide
/// any mut guarantees itself.
pub trait Database: Send + Sync {
    /// Retrieves a copy of a single, generic ent with the corresponding id
    ///
    /// This should not connect the ent back to the database upon return as
    /// that decision should be made outside of the database itself.
    fn get(&self, id: Id) -> DatabaseResult<Option<Box<dyn Ent>>>;

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
    fn insert(&self, ent: Box<dyn Ent>) -> DatabaseResult<Id>;

    /// Performs a retrieval of multiple ents of any type
    fn get_all(&self, ids: Vec<Id>) -> DatabaseResult<Vec<Box<dyn Ent>>>;

    /// Finds all generic ents that match the query
    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Box<dyn Ent>>>;
}

pub trait DatabaseExt: Database {
    /// Inserts an ent of a specific type
    fn insert_typed<E: Ent>(&self, ent: E) -> DatabaseResult<Id>;

    /// Retrieves an ent by id with a specific type
    fn get_typed<E: Ent>(&self, id: Id) -> DatabaseResult<Option<E>>;

    /// Retrieves ents by id with a specific type
    fn get_all_typed<E: Ent>(&self, ids: Vec<Id>) -> DatabaseResult<Vec<E>>;

    /// Finds ents that match the specified query and are of the specified type
    fn find_all_typed<E: Ent>(&self, query: Query) -> DatabaseResult<Vec<E>>;
}

impl<T: Database> DatabaseExt for T {
    fn insert_typed<E: Ent>(&self, ent: E) -> DatabaseResult<Id> {
        self.insert(Box::from(ent))
    }

    fn get_typed<E: Ent>(&self, id: Id) -> DatabaseResult<Option<E>> {
        self.get(id).map(|x| x.and_then(|ent| ent.to_ent::<E>()))
    }

    fn get_all_typed<E: Ent>(&self, ids: Vec<Id>) -> DatabaseResult<Vec<E>> {
        self.get_all(ids)
            .map(|x| x.into_iter().filter_map(|ent| ent.to_ent::<E>()).collect())
    }

    fn find_all_typed<E: Ent>(&self, query: Query) -> DatabaseResult<Vec<E>> {
        self.find_all(query)
            .map(|x| x.into_iter().filter_map(|ent| ent.to_ent::<E>()).collect())
    }
}

impl DatabaseExt for dyn Database {
    fn insert_typed<E: Ent>(&self, ent: E) -> DatabaseResult<Id> {
        self.insert(Box::from(ent))
    }

    fn get_typed<E: Ent>(&self, id: Id) -> DatabaseResult<Option<E>> {
        self.get(id).map(|x| x.and_then(|ent| ent.to_ent::<E>()))
    }

    fn get_all_typed<E: Ent>(&self, ids: Vec<Id>) -> DatabaseResult<Vec<E>> {
        self.get_all(ids)
            .map(|x| x.into_iter().filter_map(|ent| ent.to_ent::<E>()).collect())
    }

    fn find_all_typed<E: Ent>(&self, query: Query) -> DatabaseResult<Vec<E>> {
        self.find_all(query)
            .map(|x| x.into_iter().filter_map(|ent| ent.to_ent::<E>()).collect())
    }
}
