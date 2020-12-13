use super::{Value, ValueType};
use std::collections::HashSet;

/// Represents a definition of a field, which is comprised of its name, type
/// of value, and any associated attributes
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldDefinition {
    name: String,
    r#type: ValueType,
    attributes: Vec<FieldAttribute>,
}

impl FieldDefinition {
    /// Creates a new field definition with the given name and value and no attributes
    pub fn new<N: Into<String>, T: Into<ValueType>>(name: N, r#type: T) -> Self {
        Self::new_with_attributes(name, r#type, HashSet::new())
    }

    /// Creates a new field definition with the given name, value, and attributes
    pub fn new_with_attributes<
        N: Into<String>,
        T: Into<ValueType>,
        A: IntoIterator<Item = FieldAttribute>,
    >(
        name: N,
        r#type: T,
        attributes: A,
    ) -> Self {
        // Filter out duplicates of field attributes
        let mut attributes: Vec<FieldAttribute> = attributes.into_iter().collect();
        attributes.sort();
        attributes.dedup();

        Self {
            name: name.into(),
            r#type: r#type.into(),
            attributes,
        }
    }

    /// The name of the field tied to the definition
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The type of value of the field tied to the definition
    #[inline]
    pub fn r#type(&self) -> &ValueType {
        &self.r#type
    }

    /// The attributes associated with the field tied to the definition
    #[inline]
    pub fn attributes(&self) -> &[FieldAttribute] {
        &self.attributes
    }

    /// Returns true if this field is marked as indexed (used in databases)
    /// in its definition
    #[inline]
    pub fn is_indexed(&self) -> bool {
        self.attributes().contains(&FieldAttribute::Indexed)
    }

    /// Returns true if this field marked as immutable in its definition and
    /// should not be updated
    #[inline]
    pub fn is_immutable(&self) -> bool {
        self.attributes().contains(&FieldAttribute::Immutable)
    }
}

impl From<Field> for FieldDefinition {
    fn from(field: Field) -> Self {
        Self::new_with_attributes(field.name, field.value, field.attributes)
    }
}

/// Represents a field contained within some ent
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
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
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value of the field
    #[inline]
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Mutable value of the field
    #[inline]
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Converts field into its value
    #[inline]
    pub fn into_value(self) -> Value {
        self.value
    }

    /// The type of the value associated with the field
    pub fn to_value_type(&self) -> ValueType {
        self.value.to_type()
    }

    /// The attributes associated with the field
    #[inline]
    pub fn attributes(&self) -> &[FieldAttribute] {
        &self.attributes
    }

    /// Returns true if this field is marked as indexed (used in databases)
    #[inline]
    pub fn is_indexed(&self) -> bool {
        self.attributes().contains(&FieldAttribute::Indexed)
    }

    /// Returns true if this field marked as immutable and should not be updated
    #[inline]
    pub fn is_immutable(&self) -> bool {
        self.attributes().contains(&FieldAttribute::Immutable)
    }
}

/// Represents an attribute associated with a field for an ent
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldAttribute {
    /// Indicates that this field is indexed for faster lookup
    Indexed,

    /// Indicates that this field is immutable, meaning that it cannot be
    /// changed after being initialized
    Immutable,
}
