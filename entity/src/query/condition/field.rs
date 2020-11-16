use crate::{
    value::{Value, ValueType},
    Ent,
};
use derive_more::{Display, Error};

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
    /// Checks a loaded ent to determine if it meets the condition, returning
    /// true/false if no error is encountered
    pub fn check(&self, ent: &dyn Ent, name: &str) -> Result<bool, FieldConditionError> {
        match self {
            Self::LessThan(value) => {
                let value_type = value.to_type();
                let v = lookup_ent_field_value(ent, name, value_type)?;
                Ok(v < value)
            }
            Self::EqualTo(value) => {
                let value_type = value.to_type();
                let v = lookup_ent_field_value(ent, name, value_type)?;
                Ok(v == value)
            }
            Self::GreaterThan(value) => {
                let value_type = value.to_type();
                let v = lookup_ent_field_value(ent, name, value_type)?;
                Ok(v > value)
            }
        }
    }
}

/// Represents errors that can occur when applying a condition to an ent
#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
pub enum FieldConditionError {
    #[display(fmt = "Missing Field: {}", name)]
    MissingField { name: String },

    #[display(fmt = "Expected type {}, but got type {}", expected, actual)]
    WrongType {
        expected: ValueType,
        actual: ValueType,
    },
}

fn lookup_ent_field_value<'a>(
    ent: &'a dyn Ent,
    name: &str,
    r#type: ValueType,
) -> Result<&'a Value, FieldConditionError> {
    let value = ent
        .field_value(name)
        .ok_or_else(|| FieldConditionError::MissingField {
            name: name.to_string(),
        })?;

    if value.is_type(r#type.clone()) {
        Ok(value)
    } else {
        Err(FieldConditionError::WrongType {
            expected: r#type,
            actual: value.to_type(),
        })
    }
}
