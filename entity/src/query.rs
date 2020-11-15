use super::{Ent, Field, Value, ValueType};
use derive_more::{Constructor, Display, Error};
use std::fmt::Debug;

/// Represents a generic query to find ents within some database
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Query<T: Condition>(T);

impl<T: Condition> Query<T> {
    /// Consumes query, producing a new query with the additional condition
    /// added to the end of the conditions to be applied
    pub fn chain<U: Condition>(self, condition: U) -> Query<And<T, U>> {
        Query::new(And(self.0, condition))
    }

    /// Returns the top-level condition of the query
    pub fn condition(&self) -> &T {
        &self.0
    }

    /// Checks if the provided ent matches all conditions within the query
    pub fn check<E: Ent>(&self, ent: &E) -> Result<bool, ConditionError> {
        self.0.check(ent)
    }
}

impl Default for Query<Always> {
    /// Creates a new query that will accept all conditions
    fn default() -> Self {
        Self::new(Always)
    }
}

/// Provides helper methods on top of the Query for easier composition
pub trait QueryExt {
    type Base: Condition;

    /// Convenience method to add a new condition where the id of the ent
    /// matches the given id
    fn has_id(self, id: usize) -> Query<And<Self::Base, HasId>>;

    /// Convenience method to add a new condition where the type of the ent
    /// matches the given type
    fn has_type(self, r#type: String) -> Query<And<Self::Base, HasType>>;

    /// Convenience method to add a new condition where the ent matches
    /// both of the provided conditions
    fn and<A: Condition, B: Condition>(self, a: A, b: B) -> Query<And<Self::Base, And<A, B>>>;

    /// Convenience method to add a new condition where the ent matches
    /// either of the provided conditions
    fn or<A: Condition, B: Condition>(self, a: A, b: B) -> Query<And<Self::Base, Or<A, B>>>;

    /// Convenience method to add a new condition where the ent does not match
    /// the provided condition
    fn not<A: Condition>(self, a: A) -> Query<And<Self::Base, Not<A>>>;

    /// Convenience method to add a new condition where the specified ent
    /// field's value equals the given value
    fn ent_field_equal(self, field: impl Into<Field>) -> Query<And<Self::Base, EntFieldValue>>;

    /// Convenience method to add a new condition where the specified ent
    /// field's value is greater than the given value
    fn ent_field_greater_than(
        self,
        field: impl Into<Field>,
    ) -> Query<And<Self::Base, EntFieldValue>>;

    /// Convenience method to add a new condition where the specified ent
    /// field's value is less than the given value
    fn ent_field_less_than(self, field: impl Into<Field>) -> Query<And<Self::Base, EntFieldValue>>;
}

impl<T: Condition> QueryExt for Query<T> {
    type Base = T;

    /// Convenience method to add a new condition where the id of the ent
    /// matches the given id
    ///
    /// ```
    /// use entity::QueryExt;
    ///
    ///
    /// ```
    fn has_id(self, id: usize) -> Query<And<T, HasId>> {
        self.chain(HasId(id))
    }

    /// Convenience method to add a new condition where the type of the ent
    /// matches the given type
    fn has_type(self, r#type: String) -> Query<And<T, HasType>> {
        self.chain(HasType(r#type))
    }

    /// Convenience method to add a new condition where the ent matches
    /// both of the provided conditions
    fn and<A: Condition, B: Condition>(self, a: A, b: B) -> Query<And<T, And<A, B>>> {
        self.chain(And(a, b))
    }

    /// Convenience method to add a new condition where the ent matches
    /// either of the provided conditions
    fn or<A: Condition, B: Condition>(self, a: A, b: B) -> Query<And<T, Or<A, B>>> {
        self.chain(Or(a, b))
    }

    /// Convenience method to add a new condition where the ent does not match
    /// the provided condition
    fn not<A: Condition>(self, a: A) -> Query<And<T, Not<A>>> {
        self.chain(Not(a))
    }

    /// Convenience method to add a new condition where the specified ent
    /// field's value equals the given value
    fn ent_field_equal(self, field: impl Into<Field>) -> Query<And<T, EntFieldValue>> {
        self.chain(EntFieldValue::Equal(field.into()))
    }

    /// Convenience method to add a new condition where the specified ent
    /// field's value is greater than the given value
    fn ent_field_greater_than(self, field: impl Into<Field>) -> Query<And<T, EntFieldValue>> {
        self.chain(EntFieldValue::Greater(field.into()))
    }

    /// Convenience method to add a new condition where the specified ent
    /// field's value is less than the given value
    fn ent_field_less_than(self, field: impl Into<Field>) -> Query<And<T, EntFieldValue>> {
        self.chain(EntFieldValue::Less(field.into()))
    }
}

/// Represents a condition to a query, used to build up the query's logic
pub trait Condition: Debug + 'static {
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError>;
}

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

/// Query condition that always succeeds
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Always;

impl Condition for Always {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, Always}};
    ///
    /// let ent = MapEnt::default();
    /// let cond = Always::new();
    /// assert_eq!(cond.check(&ent), Ok(true));
    /// ```
    fn check(&self, _: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(true)
    }
}

/// Query condition that always fails
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Never;

impl Condition for Never {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, Never}};
    ///
    /// let ent = MapEnt::default();
    /// let cond = Never::new();
    /// assert_eq!(cond.check(&ent), Ok(false));
    /// ```
    fn check(&self, _: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(false)
    }
}

/// Query condition that succeeds if the ent has the specified id
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HasId(usize);

impl Condition for HasId {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, HasId}};
    ///
    /// let ent = MapEnt::empty(999);
    /// assert_eq!(HasId::new(999).check(&ent), Ok(true));
    /// assert_eq!(HasId::new(1000).check(&ent), Ok(false));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(ent.id() == self.0)
    }
}

/// Query condition that succeeds if the ent has the specified type
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HasType(String);

impl<'a> From<&'a str> for HasType {
    fn from(s: &'a str) -> Self {
        Self::new(s.to_string())
    }
}

impl Condition for HasType {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, MapEnt, query::{Condition, HasType}};
    ///
    /// let ent = MapEnt::default();
    /// assert_eq!(HasType::from(ent.r#type()).check(&ent), Ok(true));
    /// assert_eq!(HasType::from("Other").check(&ent), Ok(false));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(ent.r#type() == self.0)
    }
}

/// Query condition that succeeds if the ent succeeds with both children args
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct And<A: Condition, B: Condition>(A, B);

impl<A: Condition, B: Condition> Condition for And<A, B> {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, And, Always, Never}};
    ///
    /// let ent = MapEnt::default();
    /// assert_eq!(And::new(Always, Always).check(&ent), Ok(true));
    /// assert_eq!(And::new(Never, Always).check(&ent), Ok(false));
    /// assert_eq!(And::new(Always, Never).check(&ent), Ok(false));
    /// assert_eq!(And::new(Never, Never).check(&ent), Ok(false));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(self.0.check(ent)? && self.1.check(ent)?)
    }
}

/// Query condition that succeeds if the ent succeeds with either children arg
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Or<A: Condition, B: Condition>(A, B);

impl<A: Condition, B: Condition> Condition for Or<A, B> {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, Or, Always, Never}};
    ///
    /// let ent = MapEnt::default();
    /// assert_eq!(Or::new(Always, Always).check(&ent), Ok(true));
    /// assert_eq!(Or::new(Never, Always).check(&ent), Ok(true));
    /// assert_eq!(Or::new(Always, Never).check(&ent), Ok(true));
    /// assert_eq!(Or::new(Never, Never).check(&ent), Ok(false));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(self.0.check(ent)? || self.1.check(ent)?)
    }
}

/// Query condition that succeeds if the ent fails with the child arg
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Not<A: Condition>(A);

impl<A: Condition> Condition for Not<A> {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, query::{Condition, Not, Always, Never}};
    ///
    /// let ent = MapEnt::default();
    /// assert_eq!(Not::new(Always).check(&ent), Ok(false));
    /// assert_eq!(Not::new(Never).check(&ent), Ok(true));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        Ok(!self.0.check(ent)?)
    }
}

/// Query condition that succeeds if the ent's field succeeds with the
/// given check based on its variant
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntFieldValue {
    Less(Field),
    Equal(Field),
    Greater(Field),
}

impl EntFieldValue {
    /// Returns the name of the field whose value to compare
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Field, Value, query::EntFieldValue};
    ///
    /// let field = Field::new(String::from("name"), Value::from(false));;
    /// let cond = EntFieldValue::Less(field);
    /// assert_eq!(cond.name(), "name");
    ///
    /// let field = Field::new(String::from("name"), Value::from(false));;
    /// let cond = EntFieldValue::Equal(field);
    /// assert_eq!(cond.name(), "name");
    ///
    /// let field = Field::new(String::from("name"), Value::from(false));;
    /// let cond = EntFieldValue::Greater(field);
    /// assert_eq!(cond.name(), "name");
    /// ```
    pub fn name(&self) -> &str {
        match self {
            Self::Less(x) => x.name(),
            Self::Equal(x) => x.name(),
            Self::Greater(x) => x.name(),
        }
    }

    /// Returns the value to compare to that of the ent's field
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Field, Value, query::EntFieldValue};
    ///
    /// let field = Field::new(String::from("name"), Value::from(999));;
    /// let cond = EntFieldValue::Less(field);
    /// assert_eq!(cond.value(), &Value::from(999));
    ///
    /// let field = Field::new(String::from("name"), Value::from(999));;
    /// let cond = EntFieldValue::Equal(field);
    /// assert_eq!(cond.value(), &Value::from(999));
    ///
    /// let field = Field::new(String::from("name"), Value::from(999));;
    /// let cond = EntFieldValue::Greater(field);
    /// assert_eq!(cond.value(), &Value::from(999));
    /// ```
    pub fn value(&self) -> &Value {
        match self {
            Self::Less(x) => x.value(),
            Self::Equal(x) => x.value(),
            Self::Greater(x) => x.value(),
        }
    }
}

impl Condition for EntFieldValue {
    /// Checks an ent to verify it matches the condition
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{MapEnt, Field, Value, query::{Condition, EntFieldValue}};
    ///
    /// let ent = MapEnt::from_iter(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ].into_iter());
    ///
    /// // Check if ent's field is less than the specified value
    /// assert_eq!(EntFieldValue::Less(Field::new("name1", 100u8)).check(&ent), Ok(true));
    ///
    /// // Check if ent's field is equal to the specified value
    /// assert_eq!(EntFieldValue::Equal(Field::new("name1", 99u8)).check(&ent), Ok(true));
    ///
    /// // Check if ent's field is greater than the specified value
    /// assert_eq!(EntFieldValue::Greater(Field::new("name1", 98u8)).check(&ent), Ok(true));
    /// ```
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        let name = self.name();
        let value = ent
            .field_value(name)
            .ok_or_else(|| ConditionError::MissingField {
                name: name.to_string(),
            })?;

        let cond_value = self.value();
        if value.has_same_type(cond_value) {
            match self {
                Self::Less(_) => Ok(value < cond_value),
                Self::Equal(_) => Ok(value == cond_value),
                Self::Greater(_) => Ok(value > cond_value),
            }
        } else {
            Err(ConditionError::WrongType {
                expected: cond_value.to_type(),
                actual: value.to_type(),
            })
        }
    }
}

#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dynamic<F>(F)
where
    F: Debug + Fn(&dyn Ent) -> Result<bool, ConditionError> + 'static;

impl<F> Condition for Dynamic<F>
where
    F: Debug + Fn(&dyn Ent) -> Result<bool, ConditionError> + 'static,
{
    fn check(&self, ent: &dyn Ent) -> Result<bool, ConditionError> {
        self.0(ent)
    }
}
