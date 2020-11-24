mod database;
mod ent;

pub use database::{Database, DatabaseError, DatabaseResult};
pub use ent::*;

use derive_more::{AsMut, AsRef, Deref, DerefMut};
use std::cmp::Ordering;

/// Represents a wrapper around an ent and a connection to the database
/// containing the ent
#[derive(AsRef, AsMut, Clone, Debug, Deref, DerefMut)]
pub struct Connected<D, E>
where
    D: Database,
    E: IEnt,
{
    database: D,

    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    ent: E,
}

impl<D, E> Eq for Connected<D, E>
where
    D: Database,
    E: IEnt + Eq,
{
}

impl<D, E> PartialEq for Connected<D, E>
where
    D: Database,
    E: IEnt + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.ent == other.ent
    }
}

impl<D, E> Ord for Connected<D, E>
where
    D: Database,
    E: IEnt + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.ent.cmp(&other.ent)
    }
}

impl<D, E> PartialOrd for Connected<D, E>
where
    D: Database,
    E: IEnt + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ent.partial_cmp(&other.ent)
    }
}

impl<D, E> Connected<D, E>
where
    D: Database,
    E: IEnt,
{
    /// Returns the database found within the connection
    #[inline]
    pub fn database(&self) -> &D {
        &self.database
    }

    /// Returns the ent found within the connection
    #[inline]
    pub fn ent(&self) -> &E {
        &self.ent
    }

    /// Loads the ents associated by a specific edge
    pub fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>> {
        match self.ent().edge(name) {
            Some(e) => e
                .to_ids()
                .into_iter()
                .filter_map(|id| self.database.get(id).transpose())
                .collect(),
            None => Err(DatabaseError::MissingEdge {
                name: name.to_string(),
            }),
        }
    }
}
