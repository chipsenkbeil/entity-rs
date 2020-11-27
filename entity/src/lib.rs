mod database;
mod ent;

pub use database::*;
pub use ent::*;

/// Vendor module to re-expose relevant libraries
pub mod vendor {
    #[cfg(feature = "sled_db")]
    pub use sled;
}

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

    // CHIP CHIP CHIP
    //
    // Can we get a generic update field, update edge, add to edge, and remove
    // edge that are available within ent to be provided here? Or on IEnt?o
    //
    // Or do we just want a general update method similar to a database flush
    // that will move these changes to the database? If so, we'd want to
    // generate update methods or something for created ents
    //
    // --
    //
    // Separately, for generated ents, we would want to extend Connected for
    // any database with that specific ent via an impl so we can provide
    // explicit methods to load each type of edge and wrap in the associated
    // ent type

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
