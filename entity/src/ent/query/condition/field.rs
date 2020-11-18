use crate::ent::value::Value;

/// Represents a condition on an ent's field
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldCondition {
    /// Query condition that succeeds if the ent's field is less than the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, FieldCondition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is less than the specified value
    /// let cond = FieldCondition::LessThan(Value::from(100u8));
    /// assert_eq!(cond.check(&ent, "name1"), Ok(true));
    /// ```
    LessThan(Value),

    /// Query condition that succeeds if the ent's field is equal to the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, FieldCondition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is equal to the specified value
    /// let cond = FieldCondition::EqualTo(Value::from(99u8));
    /// assert_eq!(cond.check(&ent, "name1"), Ok(true));
    /// ```
    EqualTo(Value),

    /// Query condition that succeeds if the ent's field is greater than the
    /// specified field value
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{SchemalessEnt, Field, Value, FieldCondition};
    ///
    /// let ent = SchemalessEnt::from_collections(0, vec![
    ///     Field::new("name1", 99u8),
    ///     Field::new("name2", "some string"),
    /// ], vec![]);
    ///
    /// // Check if ent's field is greater than the specified value
    /// let cond = FieldCondition::GreaterThan(Value::from(98u8));
    /// assert_eq!(cond.check(&ent, "name1"), Ok(true));
    /// ```
    GreaterThan(Value),
}

impl FieldCondition {
    pub fn value(&self) -> &Value {
        match self {
            Self::LessThan(v) => v,
            Self::EqualTo(v) => v,
            Self::GreaterThan(v) => v,
        }
    }
}
