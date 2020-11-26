use std::fmt::Debug;

mod edge;
mod field;

pub use edge::EdgeCondition;
pub use field::{CollectionCondition, FieldCondition, ValueCondition};

/// Represents a condition to a query, used to build up the query's logic
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Condition {
    /// Query condition that always succeeds
    Always,

    /// Query condition that always fails
    Never,

    /// Query condition that succeeds if the ent has the specified id
    HasId(usize),

    /// Query condition that succeeds if the ent has the specified type
    HasType(String),

    /// Query condition that succeeds if the ent succeeds with both children args
    And(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent succeeds with either children arg
    Or(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent succeeds with only one of the children arg
    Xor(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent fails with the child arg
    Not(Box<Condition>),

    /// Query condition that succeeds if the ent's field succeeds with the
    /// given field condition
    Field(String, FieldCondition),

    /// Apply a condition on an ent's edge
    Edge(String, EdgeCondition),
}

impl std::ops::BitXor for Condition {
    type Output = Self;

    /// Shorthand to produce `Condition::Xor` by boxing the provided conditions
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Xor(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::BitAnd for Condition {
    type Output = Self;

    /// Shorthand to produce `Condition::And` by boxing the provided conditions
    fn bitand(self, rhs: Self) -> Self {
        Self::And(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::BitOr for Condition {
    type Output = Self;

    /// Shorthand to produce `Condition::Or` by boxing the provided conditions
    fn bitor(self, rhs: Self) -> Self {
        Self::Or(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Not for Condition {
    type Output = Self;

    /// Shorthand to produce `Condition::Not` by boxing the provided condition
    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

impl From<bool> for Condition {
    /// Converts bool to condition by treating true as `Condition::Always`
    /// and false as `Condition::Never`
    fn from(b: bool) -> Self {
        if b {
            Self::Always
        } else {
            Self::Never
        }
    }
}

impl From<usize> for Condition {
    /// Converts from usize to condition by treating usize as
    /// `Condition::HasId(usize)`
    fn from(id: usize) -> Self {
        Self::HasId(id)
    }
}

impl From<String> for Condition {
    /// Converts from string to condition by treating string as
    /// `Condition::HasType(String)`
    fn from(r#type: String) -> Self {
        Self::HasType(r#type)
    }
}

impl<'a> From<&'a str> for Condition {
    /// Converts from &str to condition by allocating new string and treating
    /// as `Condition::HasType(String)`
    fn from(r#type: &'a str) -> Self {
        Self::from(r#type.to_string())
    }
}
