#[cfg(not(feature = "async"))]
pub use sync::*;

#[cfg(feature = "async")]
pub use r#async::*;

use super::Ent;

#[cfg(not(feature = "async"))]
mod sync {
    use super::Ent;

    /// Represents a synchronous context, which can access some underlying
    /// database to perform CRUD operations using ents
    pub trait Context {
        /// Retrieves a single, generic ent with the corresponding id
        fn get(&self, id: usize) -> Option<&dyn Ent>;

        /// Retrieves a collection of generic ents with the corresponding ids
        fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> Vec<&dyn Ent>;

        /// Removes the ent with the corresponding id, triggering edge
        /// processing for all disconnected ents. If indicated, this will
        /// adjust ents connected with this ent via edges, removing any if
        /// indicated.
        fn remove(&mut self, id: usize, adjust_edges: bool) -> Option<Box<dyn Ent>>;

        /// Inserts a new ent using its id as the primary index, overwriting
        /// any ent with a matching id. If indicated, this will adjust ents
        /// connected with this ent via edges, removing any if indicated.
        fn insert(&mut self, ent: impl Ent, adjust_edges: bool) -> Option<Box<dyn Ent>>;

        /// Performs a retrieval of a single ent and attempts to cast it
        /// to the specified concrete type
        fn get_typed<E: Ent>(&self, id: usize) -> Option<&E> {
            self.get(id)
                .and_then(|ent| ent.as_any().downcast_ref::<E>())
        }

        /// Performs a retrieval of multiple ents and attempts to cast them
        /// to the specified concrete type
        fn get_all_typed<E: Ent>(&self, ids: impl IntoIterator<Item = usize>) -> Vec<&E> {
            self.get_all(ids)
                .into_iter()
                .filter_map(|ent| ent.as_any().downcast_ref::<E>())
                .collect()
        }
    }
}

#[cfg(feature = "async")]
mod r#async {
    use super::Ent;

    /// Represents an asynchronous context, which can access some underlying
    /// database to perform CRUD operations using ents
    ///
    /// Note that this requires use of `async_trait` declaratively on all
    /// implementations of this context
    #[async_trait::async_trait]
    pub trait Context {}
}
