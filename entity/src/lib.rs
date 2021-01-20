#![forbid(unsafe_code)]

mod alloc;
mod any;
mod database;
mod ent;
pub mod ext;
pub mod global;

pub use alloc::{Id, IdAllocator, EPHEMERAL_ID};
pub use any::*;
pub use database::*;
pub use ent::*;

#[cfg(feature = "macros")]
pub use entity_macros::*;

/// Represents a collection of core traits that should be imported to enable
/// methods on a variety of types such as [`Ent`] and [`EntBuilder`]
pub mod prelude {
    pub use super::{
        AsAny, Database, DatabaseExt, Ent, EntBuilder, EntExt, EntLoader, EntQuery, EntType,
        EntWrapper,
    };
}

/// Vendor module to re-expose relevant libraries
pub mod vendor {
    #[cfg(feature = "sled_db")]
    pub use ::sled;

    /// Re-exported macros, useful only to [`entity_macros`] crate
    pub mod macros {
        /// Re-export of serde
        pub mod serde {
            /// Indicates whether or not the included serde derive macros are
            /// the result of the feature existing (true) or a no-op (false)
            #[inline]
            pub const fn exists() -> bool {
                cfg!(feature = "serde-1")
            }

            #[cfg(feature = "serde-1")]
            pub use ::serde::Serialize;

            #[cfg(feature = "serde-1")]
            pub use ::serde::Deserialize;

            #[cfg(not(feature = "serde-1"))]
            pub use ::entity_noop_macros::NoopDeriveSerde as Serialize;

            #[cfg(not(feature = "serde-1"))]
            pub use ::entity_noop_macros::NoopDeriveSerde as Deserialize;
        }

        /// Re-export of typetag
        pub mod typetag {
            /// Indicates whether or not the included typetag attr macro is the
            /// result of the feature existing (true) or a no-op (false)
            #[inline]
            pub const fn exists() -> bool {
                cfg!(feature = "typetag")
            }

            #[cfg(feature = "typetag")]
            pub use ::typetag::serde;

            #[cfg(not(feature = "typetag"))]
            pub use ::entity_noop_macros::noop_attr as serde;
        }
    }
}
