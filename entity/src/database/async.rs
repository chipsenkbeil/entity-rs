use async_trait::async_trait;

/// Represents an asynchronous database, which performs non-blocking CRUD
/// operations using ents. All operations only require a reference to the
/// database and it is up to each implementation to provide proper
/// locking and safeguards to ensure that multi-threaded access does
/// not cause problems.
///
/// Implementors of this trait will be required to annotate using
/// `#[async_trait]` due to limitations in rust for traits with
/// async methods.
#[async_trait]
pub trait Database {
    /// Retrieves a single, generic ent with the corresponding id
    async fn get(&self, id: usize) -> DatabaseResult<Option<&dyn Ent>>;

    /// Finds all generic ents that match the query
    async fn find_all(&self, query: Query) -> DatabaseResult<Vec<&dyn Ent>>;

    /// Removes the ent with the corresponding id, triggering edge
    /// processing for all disconnected ents. If indicated, this will
    /// adjust ents connected with this ent via edges, removing any if
    /// indicated.
    async fn remove(&self, id: usize, adjust_edges: bool) -> DatabaseResult<()>;

    /// Inserts a new ent using its id as the primary index, overwriting
    /// any ent with a matching id.
    async fn insert(&self, ent: impl Ent) -> DatabaseResult<()>;

    /// Performs a retrieval of a single ent and attempts to cast it
    /// to the specified concrete type
    async fn get_typed<E: Ent>(&self, id: usize) -> DatabaseResult<Option<&E>> {
        self.get(id)
            .await
            .map(|maybe_ent| maybe_ent.and_then(|ent| ent.as_any().downcast_ref::<E>()))
    }

    /// Performs a search for all ents that match the given query and
    /// attempts to cast them to the specified concrete type
    async fn find_all_typed<E: Ent>(&self, query: Query) -> DatabaseResult<Vec<&E>> {
        self.find_all(query).await.map(|ents| {
            ents.into_iter()
                .filter_map(|ent| ent.as_any().downcast_ref::<E>())
                .collect()
        })
    }
}

/// Represents extensions to the database to provide advanced functionality.
///
/// Implementors of this trait will be required to annotate using
/// `#[async_trait]` due to limitations in rust for traits with
/// async methods.
#[async_trait]
pub trait DatabaseExt {
    /// Performs a retrieval of multiple ents
    async fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> DatabaseResult<Vec<&dyn Ent>>;

    /// Performs a retrieval of multiple ents and attempts to cast them
    /// to the specified concrete type
    async fn get_all_typed<E: Ent>(
        &self,
        ids: impl IntoIterator<Item = usize>,
    ) -> DatabaseResult<Vec<&E>> {
        self.get_all(ids).await.map(|ents| {
            ents.into_iter()
                .filter_map(|ent| ent.as_any().downcast_ref::<E>())
                .collect()
        })
    }
}

/// Blanket implementation of extended database functionality. For specific
/// implementations, this can be overridden to use more performant and
/// exclusive means to do each operation.
#[async_trait]
impl<T: Database> DatabaseExt for T {
    /// Performs a retrieval of multiple ents
    async fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> DatabaseResult<Vec<&dyn Ent>> {
        ids.into_iter()
            .filter_map(|id| self.get(id).await.transpose())
            .collect()
    }
}

/// Asynchronous implementation of ConnectedEnt for use with asynchronous databases
impl<C, E> ConnectedEnt<C, E>
where
    C: Database + 'static,
    E: Ent,
{
    /// Loads the ents connected by the named edge
    pub async fn load_edge(&self, name: &str) -> ConnectedEntResult<Vec<&dyn Ent>> {
        match self.edge(name) {
            Some(e) => self
                .connection
                .get_all(e.to_ids())
                .await
                .map_err(ConnectedEntError::Database),
            None => Err(ConnectedEntError::MissingEdge {
                name: name.to_string(),
            }),
        }
    }

    /// Loads the ents connected by the named edge and attempts to cast them
    /// to the specified concrete type
    pub async fn load_edge_typed<T: Ent>(&self, name: &str) -> ConnectedEntResult<Vec<&T>> {
        self.load_edge(name).await.map(|ents| {
            ents.into_iter()
                .filter_map(|ent| ent.as_any().downcast_ref::<T>())
                .collect()
        })
    }
}
