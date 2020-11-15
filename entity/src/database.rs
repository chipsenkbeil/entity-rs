#[cfg(not(feature = "async"))]
pub use sync::*;

#[cfg(feature = "async")]
pub use r#async::*;

use super::{Condition, Ent, Query};
use derive_more::{Display, Error};

/// Alias to a result that can contain a database error
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents some error the can occur when accessing the database
#[derive(Debug, Display, Error)]
pub enum DatabaseError {
    Get { msg: String },
    GetAll { msg: String },
    FindAll { msg: String },
    Insert { msg: String },
    Remove { msg: String },
}

#[cfg(not(feature = "async"))]
mod sync {
    use super::*;

    /// Represents a synchronous database, which performs blocking CRUD
    /// operations using ents
    pub trait Database {
        /// Retrieves a single, generic ent with the corresponding id
        fn get(&self, id: usize) -> DatabaseResult<Option<&dyn Ent>>;

        /// Retrieves a collection of generic ents with the corresponding ids
        fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> DatabaseResult<Vec<&dyn Ent>>;

        /// Finds all generic ents that match the corresponding query
        fn find_all<T: Condition>(&self, query: Query<T>) -> DatabaseResult<Vec<&dyn Ent>>;

        /// Removes the ent with the corresponding id, triggering edge
        /// processing for all disconnected ents. If indicated, this will
        /// adjust ents connected with this ent via edges, removing any if
        /// indicated.
        fn remove(&mut self, id: usize, adjust_edges: bool) -> DatabaseResult<()>;

        /// Inserts a new ent using its id as the primary index, overwriting
        /// any ent with a matching id. If indicated, this will adjust ents
        /// connected with this ent via edges, removing any if indicated.
        fn insert(&mut self, ent: impl Ent, adjust_edges: bool) -> DatabaseResult<()>;

        /// Performs a retrieval of a single ent and attempts to cast it
        /// to the specified concrete type
        fn get_typed<E: Ent>(&self, id: usize) -> DatabaseResult<Option<&E>> {
            self.get(id)
                .map(|maybe_ent| maybe_ent.and_then(|ent| ent.as_any().downcast_ref::<E>()))
        }

        /// Performs a retrieval of multiple ents and attempts to cast them
        /// to the specified concrete type
        fn get_all_typed<E: Ent>(
            &self,
            ids: impl IntoIterator<Item = usize>,
        ) -> DatabaseResult<Vec<&E>> {
            self.get_all(ids).map(|ents| {
                ents.into_iter()
                    .filter_map(|ent| ent.as_any().downcast_ref::<E>())
                    .collect()
            })
        }

        /// Performs a search for all ents that match the given query and
        /// attempts to cast them to the specified concrete type
        fn find_all_typed<T: Condition, E: Ent>(&self, query: Query<T>) -> DatabaseResult<Vec<&E>> {
            self.find_all(query).map(|ents| {
                ents.into_iter()
                    .filter_map(|ent| ent.as_any().downcast_ref::<E>())
                    .collect()
            })
        }
    }
}

#[cfg(feature = "async")]
mod r#async {
    use super::*;

    /// Represents an asynchronous database, which performs non-blocking CRUD
    /// operations using ents
    ///
    /// Note that this requires use of `async_trait` declaratively on all
    /// implementations of this database
    #[async_trait::async_trait]
    pub trait Database {}
}
