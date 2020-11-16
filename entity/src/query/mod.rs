use derive_more::Constructor;
use std::fmt::Debug;

mod condition;
pub use condition::*;

/// Represents a generic query to find ents within some database
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Query(Condition);

impl Query {
    /// Consumes query, producing a new query with the additional condition
    /// added to the end of the conditions to be applied, essentially
    /// performing the same as `query.condition() & condition`
    pub fn chain(self, condition: Condition) -> Self {
        Query::new(self.0 & condition)
    }

    /// Returns the top-level condition of the query
    pub fn condition(&self) -> &Condition {
        &self.0
    }
}

impl Default for Query {
    /// Creates a new query that will accept all conditions
    fn default() -> Self {
        Self::new(Condition::Always)
    }
}

/// Provides helper methods on top of the Query for easier composition
pub trait QueryExt {
    /// Convenience method to add a new condition where the id of the ent
    /// matches the given id
    fn has_id(self, id: usize) -> Query;

    /// Convenience method to add a new condition where the type of the ent
    /// matches the given type
    fn has_type(self, r#type: impl Into<String>) -> Query;

    /// Convenience method to add a new condition where the ent matches
    /// both of the provided conditions
    fn and(self, a: Condition, b: Condition) -> Query;

    /// Convenience method to add a new condition where the ent matches
    /// either of the provided conditions
    fn or(self, a: Condition, b: Condition) -> Query;

    /// Convenience method to add a new condition where the ent does not match
    /// the provided condition
    fn not(self, cond: Condition) -> Query;

    /// Convenience method to add a new condition for an ent's field
    fn field(self, name: impl Into<String>, cond: FieldCondition) -> Query;
}

impl QueryExt for Query {
    /// Convenience method to add a new condition where the id of the ent
    /// matches the given id
    fn has_id(self, id: usize) -> Query {
        self.chain(Condition::HasId(id))
    }

    /// Convenience method to add a new condition where the type of the ent
    /// matches the given type
    fn has_type(self, r#type: impl Into<String>) -> Query {
        self.chain(Condition::HasType(r#type.into()))
    }

    /// Convenience method to add a new condition where the ent matches
    /// both of the provided conditions
    fn and(self, a: Condition, b: Condition) -> Query {
        self.chain(a & b)
    }

    /// Convenience method to add a new condition where the ent matches
    /// either of the provided conditions
    fn or(self, a: Condition, b: Condition) -> Query {
        self.chain(a | b)
    }

    /// Convenience method to add a new condition where the ent does not match
    /// the provided condition
    fn not(self, condition: Condition) -> Query {
        self.chain(!condition)
    }

    /// Convenience method to add a new condition for an ent's field
    fn field(self, name: impl Into<String>, cond: FieldCondition) -> Query {
        self.chain(Condition::Field(name.into(), cond))
    }
}
