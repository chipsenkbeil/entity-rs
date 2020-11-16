use crate::Ent;
use derive_more::{Display, Error, From};
use std::fmt::Debug;

mod edge;
mod field;

pub use field::{FieldCondition, FieldConditionError};

#[derive(Clone, Debug, Display, Error, From, PartialEq, Eq)]
pub enum ConditionError {
    Field(FieldConditionError),
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

    /// Query condition that succeeds if the ent's field succeeds with the
    /// given field condition
    ///
    /// ## Examples
    ///
    /// ```
    /// ```
    Field(String, FieldCondition),
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
            Self::Field(name, cond) => Ok(cond.check(ent, name).map_err(ConditionError::from)?),
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
