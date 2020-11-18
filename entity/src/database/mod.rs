#[cfg(feature = "async")]
mod r#async;

#[cfg(not(feature = "async"))]
mod sync;

#[cfg(feature = "async")]
pub use r#async::*;

#[cfg(not(feature = "async"))]
pub use sync::*;

use derive_more::Display;

/// Alias to a result that can contain a database error
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents some error the can occur when accessing the database
#[derive(Debug, Display)]
pub enum DatabaseError {
    #[display(fmt = "Connection Issue: {}", source)]
    Connection { source: Box<dyn std::error::Error> },

    #[display(fmt = "Missing Field: {}", name)]
    MissingField { name: String },

    #[display(fmt = "Missing Edge: {}", name)]
    MissingEdge { name: String },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: crate::ent::ValueType,
        actual: crate::ent::ValueType,
    },
}

impl std::error::Error for DatabaseError {}
