use super::{Value, ValueType};
use std::collections::HashSet;

/// Represents a field contained within some ent
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
    name: String,
    value: Value,
    attributes: Vec<FieldAttribute>,
}

impl Field {
    /// Creates a new field with the given name and value and no attributes
    pub fn new<N: Into<String>, V: Into<Value>>(name: N, value: V) -> Self {
        Self::new_with_attributes(name, value, HashSet::new())
    }

    /// Creates a new field with the given name, value, and attributes
    pub fn new_with_attributes<
        N: Into<String>,
        V: Into<Value>,
        A: IntoIterator<Item = FieldAttribute>,
    >(
        name: N,
        value: V,
        attributes: A,
    ) -> Self {
        // Filter out duplicates of field attributes
        let mut attributes: Vec<FieldAttribute> = attributes.into_iter().collect();
        attributes.sort();
        attributes.dedup();

        Self {
            name: name.into(),
            value: value.into(),
            attributes,
        }
    }

    /// The name of the field
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value of the field
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Converts field into its value
    pub fn into_value(self) -> Value {
        self.value
    }

    /// The type of the value associated with the field
    pub fn to_value_type(&self) -> ValueType {
        self.value.to_type()
    }

    pub fn attributes(&self) -> &[FieldAttribute] {
        &self.attributes
    }
}

/// Represents an attribute associated with a field for an ent
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldAttribute {
    /// Indicates that this field is indexed for faster lookup
    Indexed,
}
