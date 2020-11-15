use super::{Ent, Field, Value, ValueType};
use derive_more::{Constructor, Display, Error};
use std::fmt::Debug;

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
    fn not(self, a: Condition) -> Query;

    /// Convenience method to add a new condition where the specified ent
    /// field's value equals the given value
    fn ent_field_equal_to(self, field: impl Into<Field>) -> Query;

    /// Convenience method to add a new condition where the specified ent
    /// field's value is greater than the given value
    fn ent_field_greater_than(self, field: impl Into<Field>) -> Query;

    /// Convenience method to add a new condition where the specified ent
    /// field's value is less than the given value
    fn ent_field_less_than(self, field: impl Into<Field>) -> Query;
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

    /// Convenience method to add a new condition where the specified ent
    /// field's value equals the given value
    fn ent_field_equal_to(self, field: impl Into<Field>) -> Query {
        self.chain(Condition::EntFieldEqualTo(field.into()))
    }

    /// Convenience method to add a new condition where the specified ent
    /// field's value is greater than the given value
    fn ent_field_greater_than(self, field: impl Into<Field>) -> Query {
        self.chain(Condition::EntFieldGreaterThan(field.into()))
    }

    /// Convenience method to add a new condition where the specified ent
    /// field's value is less than the given value
    fn ent_field_less_than(self, field: impl Into<Field>) -> Query {
        self.chain(Condition::EntFieldLessThan(field.into()))
    }
}

/// Represents errors that can occur when applying a condition to an ent
#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
pub enum ConditionError {
    #[display(fmt = "Missing Field: {}", name)]
    MissingField { name: String },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: ValueType,
        actual: ValueType,
    },
}

/// Represents a condition to a query, used to build up the query's logic
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Condition {
    /// Query condition that always succeeds
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// let cond = Condition::Always;
    /// assert_eq!(cond.check(&ent), Ok(true));
    /// ```
    Always,

    /// Query condition that always fails
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// let cond = Condition::Never;
    /// assert_eq!(cond.check(&ent), Ok(false));
    /// ```
    Never,

    /// Query condition that succeeds if the ent has the specified id
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::empty(999);
    /// assert_eq!(Condition::HasId(999).check(&ent), Ok(true));
    /// assert_eq!(Condition::HasId(1000).check(&ent), Ok(false));
    /// ```
    HasId(usize),

    /// Query condition that succeeds if the ent has the specified type
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(Condition::HasType(ent.r#type().to_string()).check(&ent), Ok(true));
    /// assert_eq!(Condition::HasType("Other".to_string()).check(&ent), Ok(false));
    /// ```
    HasType(String),

    /// Query condition that succeeds if the ent succeeds with both children args
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(Condition::And(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::And(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(false));
    /// assert_eq!(Condition::And(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(false));
    /// assert_eq!(Condition::And(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(false));
    /// ```
    And(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent succeeds with either children arg
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(Condition::Or(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::Or(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::Or(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::Or(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(false));
    /// ```
    Or(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent succeeds with only one of the children arg
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(Condition::Xor(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(false));
    /// assert_eq!(Condition::Xor(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Always)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::Xor(
    ///     Box::from(Condition::Always),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(true));
    /// assert_eq!(Condition::Xor(
    ///     Box::from(Condition::Never),
    ///     Box::from(Condition::Never)
    /// ).check(&ent), Ok(false));
    /// ```
    Xor(Box<Condition>, Box<Condition>),

    /// Query condition that succeeds if the ent fails with the child arg
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Condition};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(Condition::Not(Box::from(Condition::Always)).check(&ent), Ok(false));
    /// assert_eq!(Condition::Not(Box::from(Condition::Never)).check(&ent), Ok(true));
    /// ```
    Not(Box<Condition>),

    /// Query condition that succeeds if the ent's field is less than the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, Condition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is less than the specified value
    /// let cond = Condition::EntFieldLessThan(Field::new("name1", 100u8));
    /// assert_eq!(cond.check(&ent), Ok(true));
    /// ```
    EntFieldLessThan(Field),

    /// Query condition that succeeds if the ent's field is equal to the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, Condition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is equal to the specified value
    /// let cond = Condition::EntFieldEqualTo(Field::new("name1", 99u8));
    /// assert_eq!(cond.check(&ent), Ok(true));
    /// ```
    EntFieldEqualTo(Field),

    /// Query condition that succeeds if the ent's field is greater than the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, Condition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is greater than the specified value
    /// let cond = Condition::EntFieldGreaterThan(Field::new("name1", 98u8));
    /// assert_eq!(cond.check(&ent), Ok(true));
    /// ```
    EntFieldGreaterThan(Field),
}

impl Condition {
    /// Checks a loaded ent to determine if it meets the condition, returning
    /// true/false if no error is encountered
    pub fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        match self {
            Self::Always => Ok(true),
            Self::Never => Ok(false),
            Self::HasId(id) => Ok(ent.id() == *id),
            Self::HasType(r#type) => Ok(ent.r#type() == r#type),
            Self::And(a, b) => Ok(a.check(ent)? && b.check(ent)?),
            Self::Or(a, b) => Ok(a.check(ent)? || b.check(ent)?),
            Self::Xor(a, b) => Ok(matches!(
                (a.check(ent)?, b.check(ent)?),
                (true, false) | (false, true)
            )),
            Self::Not(cond) => Ok(!cond.check(ent)?),
            Self::EntFieldLessThan(f) => {
                let v = lookup_ent_field_value(ent, f.name(), f.value().to_type())?;
                Ok(v < f.value())
            }
            Self::EntFieldEqualTo(f) => {
                let v = lookup_ent_field_value(ent, f.name(), f.value().to_type())?;
                Ok(v == f.value())
            }
            Self::EntFieldGreaterThan(f) => {
                let v = lookup_ent_field_value(ent, f.name(), f.value().to_type())?;
                Ok(v > f.value())
            }
        }
    }
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

fn lookup_ent_field_value<'a>(
    ent: &'a dyn Ent,
    name: &str,
    r#type: ValueType,
) -> Result<&'a Value, ConditionError> {
    let value = ent
        .field_value(name)
        .ok_or_else(|| ConditionError::MissingField {
            name: name.to_string(),
        })?;

    if value.is_type(r#type.clone()) {
        Ok(value)
    } else {
        Err(ConditionError::WrongType {
            expected: r#type,
            actual: value.to_type(),
        })
    }
}
