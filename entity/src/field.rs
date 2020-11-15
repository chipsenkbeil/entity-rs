use super::{Value, ValueType};
use derive_more::Into;
use std::collections::HashSet;

/// Represents a field contained within some ent
#[derive(Clone, Debug, PartialEq, Eq, Into)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
    name: String,
    value: Value,
}

impl Field {
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
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
}

/// Represents a field definition for an ent
#[derive(Clone, Debug, PartialEq, Eq, Into)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldDefinition {
    name: String,
    r#type: ValueType,
    attributes: HashSet<FieldDefinitionAttribute>,
}

/// Represents an attribute associated with a field definition for an ent
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldDefinitionAttribute {
    Indexed,
}

impl FieldDefinition {
    /// Creates a new field definition for use by a database
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{
    ///     FieldDefinition as FD,
    ///     FieldDefinitionAttribute as FDA,
    ///     PrimitiveValueType as PVT
    /// };
    ///
    /// let fd = FD::new("my field", PVT::U32, vec![FDA::Indexed]);
    /// assert_eq!(fd.name(), "my field");
    /// assert_eq!(fd.r#type(), PVT::U32);
    /// assert_eq!(fd.attributes(), vec![FDA::Indexed].into_iter().collect());
    /// ```
    pub fn new(
        name: impl Into<String>,
        r#type: impl Into<ValueType>,
        attributes: impl IntoIterator<Item = FieldDefinitionAttribute>,
    ) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            attributes: attributes.into_iter().collect(),
        }
    }

    /// The name of the field
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The type associated with the field
    pub fn r#type(&self) -> ValueType {
        self.r#type
    }

    /// Attributes associated with the definition
    pub fn attributes(&self) -> HashSet<FieldDefinitionAttribute> {
        self.attributes.iter().copied().collect()
    }
}
