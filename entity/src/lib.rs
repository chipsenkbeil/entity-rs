mod alloc;
mod database;
mod ent;

pub use alloc::{Id, IdAllocator, EPHEMERAL_ID};
pub use database::*;
pub use ent::*;

/// Vendor module to re-expose relevant libraries
pub mod vendor {
    #[cfg(feature = "sled_db")]
    pub use sled;

    #[cfg(feature = "serde")]
    pub use serde;

    #[cfg(feature = "typetag")]
    pub use typetag;
}
