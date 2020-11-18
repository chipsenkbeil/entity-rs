mod any;
mod database;
mod ent;

pub use any::AsAny;
pub use database::{Database, DatabaseError, DatabaseResult};
pub use ent::*;

/// Represents the interface for a generic entity whose fields and edges
/// can be accessed by str name regardless of compile-time characteristics
///
/// Based on https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf
pub trait IEnt: AsAny {
    /// Represents the unique id associated with each entity instance
    fn id(&self) -> usize;

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    fn r#type<'a>(&'a self) -> &'a str;

    /// Represents the time when the instance of the ent was created
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn created(&self) -> u64;

    /// Represents the time when the instance of the ent was last updated
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn last_updated(&self) -> u64;

    /// Represents fields contained within the ent instance, guaranteeing
    /// that fields are unique by name
    fn fields(&self) -> Vec<&Field>;

    /// Retrieves the field with the provided name within the ent instance
    fn field(&self, name: &str) -> Option<&Field>;

    /// Retrieves the value for the field with the provided name within the
    /// ent instance
    fn field_value(&self, name: &str) -> Option<&Value> {
        self.field(name).map(|f| f.value())
    }

    /// Represents edges between the ent instance and some referred ents
    fn edges(&self) -> Vec<&Edge>;

    /// Retrieves the edge with the provided name within the ent instance
    fn edge(&self, name: &str) -> Option<&Edge>;
}

/// Represents the interface to expose the connection to a database for
/// some ent
pub trait Connected: IEnt {
    type DB: Database;

    /// Returns the connection to the database for some entity
    fn connection(&self) -> &Self::DB;
}

/// Blanket implementation for all ents that enables them to be converted
/// to any, which is useful when converting `&dyn Ent` into a concrete type
impl<T: IEnt> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Represents an ent that is connected to a synchronous database and is
/// able to load ents for its edges directly
#[cfg(not(feature = "async"))]
pub trait ConnectedExt: Connected {
    /// Loads the ents connected by the named edge
    fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>>;
}

#[cfg(not(feature = "async"))]
impl<T: Connected> ConnectedExt for T {
    fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>> {
        match self.edge(name) {
            Some(e) => e
                .to_ids()
                .into_iter()
                .filter_map(|id| self.connection().get(id).transpose())
                .collect(),
            None => Err(DatabaseError::MissingEdge {
                name: name.to_string(),
            }),
        }
    }
}

/// Represents an ent that is connected to an asynchronous database and is
/// able to load ents for its edges directly
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait ConnectedEnt: IEnt + Connected {
    /// Loads the ents connected by the named edge
    async fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>>;
}

#[cfg(feature = "async")]
impl<T: Connected> ConnectedExt for T {
    async fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>> {
        match self.edge(name) {
            Some(e) => e
                .to_ids()
                .into_iter()
                .filter_map(|id| self.connection().get(id).await.transpose())
                .collect(),
            None => Err(DatabaseError::MissingEdge {
                name: name.to_string(),
            }),
        }
    }
}
