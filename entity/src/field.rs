use super::{Value, ValueType};
use derive_more::Into;

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

    pub fn name(&self) -> &str {
        &self.name
    }

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
}

impl FieldDefinition {
    pub fn new(name: impl Into<String>, r#type: ValueType) -> Self {
        Self {
            name: name.into(),
            r#type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> ValueType {
        self.r#type
    }
}
