use std::collections::HashSet;
use strum::{Display, EnumDiscriminants, EnumString};

/// Represents a edge from an ent to one or more other ents
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edge {
    name: String,
    value: EdgeValue,
}

impl Edge {
    pub fn new(name: impl Into<String>, value: EdgeValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// The name of the edge
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value of the edge
    pub fn value(&self) -> &EdgeValue {
        &self.value
    }

    /// Converts to the ids of the ents referenced by this edge
    pub fn to_ids(&self) -> Vec<usize> {
        self.value.to_ids()
    }
}

/// Represents the value of an edge, which is some collection of ent ids
#[derive(Clone, Debug, PartialEq, Eq, EnumDiscriminants)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(Display, EnumString))]
#[strum_discriminants(name(EdgeValueType), strum(serialize_all = "snake_case"))]
#[cfg_attr(
    feature = "serde",
    strum_discriminants(derive(serde::Serialize, serde::Deserialize))
)]
pub enum EdgeValue {
    /// Many instances of current ent refer to many instances of the referred ent
    ManyToMany(Vec<usize>),
    /// Many instances of current ent refer to one instance of the referred ent
    ManyToOne(usize),
    /// One instance of current ent refers to many instances of the referred ent
    OneToMany(Vec<usize>),
    /// One instance of current ent refers to one instance of the referred ent
    OneToOne(usize),
}

impl EdgeValue {
    /// Produces all ids of ents referenced by this edge's value
    pub fn to_ids(&self) -> Vec<usize> {
        match self {
            Self::ManyToMany(x) => x.clone(),
            Self::ManyToOne(x) => vec![*x],
            Self::OneToMany(x) => x.clone(),
            Self::OneToOne(x) => vec![*x],
        }
    }
}

/// Represents an edge definition for an ent
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeDefinition {
    name: String,
    r#type: EdgeValueType,
    attributes: HashSet<EdgeDefinitionAttribute>,
}

/// Represents an attribute associated with an edge definition for an ent
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeDefinitionAttribute {
    /// When this ent instance is deleted, delete the reverse edge connections
    /// of all ents connected by this edge
    ShallowDelete,

    /// When this ent instance is deleted, fully delete all ents connected
    /// by this edge
    DeepDelete,
}

impl EdgeDefinition {
    /// Creates a new edge definition for use by a database
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{
    ///     EdgeDefinition as ED,
    ///     EdgeDefinitionAttribute as EDA,
    ///     EdgeValueType as EVT,
    /// };
    ///
    /// let ed = ED::new("my edge", EVT::OneToOne, vec![EDA::ShallowDelete]);
    /// assert_eq!(ed.name(), "my edge");
    /// assert_eq!(ed.r#type(), EVT::OneToOne);
    /// assert_eq!(ed.attributes(), vec![EDA::ShallowDelete].into_iter().collect());
    /// ```
    pub fn new(
        name: impl Into<String>,
        r#type: impl Into<EdgeValueType>,
        attributes: impl IntoIterator<Item = EdgeDefinitionAttribute>,
    ) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            attributes: attributes.into_iter().collect(),
        }
    }

    /// The name of the edge
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The type associated with the edge
    pub fn r#type(&self) -> EdgeValueType {
        self.r#type
    }

    /// Attributes associated with the definition
    pub fn attributes(&self) -> HashSet<EdgeDefinitionAttribute> {
        self.attributes.iter().copied().collect()
    }
}
