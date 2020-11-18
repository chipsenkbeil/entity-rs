use super::{Value, ValueType};
use std::collections::HashSet;

/// Represents a field contained within some ent
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldDefinition {
    name: String,
    r#type: ValueType,
    default_value: Option<Value>,
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
    ///     Value as V,
    ///     ValueType as VT,
    ///     PrimitiveValue as PV,
    ///     PrimitiveValueType as PVT
    /// };
    ///
    /// let fd = FD::new("my field", PVT::U32, Some(5u32), vec![FDA::Indexed]);
    /// assert_eq!(fd.name(), "my field");
    /// assert_eq!(fd.r#type(), &VT::Primitive(PVT::U32));
    /// assert_eq!(fd.default_value(), Some(&V::Primitive(PV::U32(5))));
    /// assert_eq!(fd.attributes(), vec![FDA::Indexed].into_iter().collect());
    /// ```
    pub fn new(
        name: impl Into<String>,
        r#type: impl Into<ValueType>,
        default_value: Option<impl Into<Value>>,
        attributes: impl IntoIterator<Item = FieldDefinitionAttribute>,
    ) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            default_value: default_value.map(Into::into),
            attributes: attributes.into_iter().collect(),
        }
    }

    /// The name of the field
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The type associated with the field
    pub fn r#type(&self) -> &ValueType {
        &self.r#type
    }

    /// The default value to apply when creating new ents with this field
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    /// Attributes associated with the definition
    pub fn attributes(&self) -> HashSet<FieldDefinitionAttribute> {
        self.attributes.iter().copied().collect()
    }
}
