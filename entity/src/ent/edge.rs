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

    /// The mutable value of the edge
    pub fn value_mut(&mut self) -> &mut EdgeValue {
        &mut self.value
    }

    /// Converts to the ids of the ents referenced by this edge
    pub fn to_ids(&self) -> Vec<usize> {
        self.value.to_ids()
    }

    /// Converts to the edge's value type
    pub fn to_type(&self) -> EdgeValueType {
        self.value().into()
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
    /// Edge can potentially have one outward connection
    MaybeOne(Option<usize>),
    /// Edge can have exactly one outward connection
    One(usize),
    /// Edge can have many outward connections
    Many(Vec<usize>),
}

/// Represents some error the can occur when mutating an edge's value
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum EdgeValueMutationError {
    #[display(fmt = "Too many ids for edge of type {}: {}", r#type, cnt)]
    TooManyIds { r#type: EdgeValueType, cnt: usize },

    #[display(fmt = "Too ids ents for edge of type {}: {}", r#type, cnt)]
    TooFewIds { r#type: EdgeValueType, cnt: usize },

    #[display(fmt = "Change invalidates edge of type {}", r#type)]
    InvalidatesEdge { r#type: EdgeValueType },
}

impl EdgeValue {
    /// Produces all ids of ents referenced by this edge's value
    pub fn to_ids(&self) -> Vec<usize> {
        match self {
            Self::MaybeOne(x) => x.into_iter().copied().collect(),
            Self::One(x) => vec![*x],
            Self::Many(x) => x.clone(),
        }
    }

    pub fn to_type(&self) -> EdgeValueType {
        self.into()
    }

    /// Adds the provided ids to the edge value, failing if the ids would
    /// exceed the maximum allowed by the edge with current ids included
    pub fn add_ids(
        &mut self,
        into_ids: impl IntoIterator<Item = usize>,
    ) -> Result<(), EdgeValueMutationError> {
        let ids = into_ids.into_iter().collect::<HashSet<usize>>();
        let cnt = self.id_count();

        // Fails if adding these ids would exceed the maximum allowed ids
        if cnt + ids.len() > self.max_ids_allowed() {
            return Err(EdgeValueMutationError::TooManyIds {
                r#type: self.to_type(),
                cnt: ids.len(),
            });
        }

        // Update our optional id as we know it should be None and that we
        // only were given a single id
        if let Self::MaybeOne(maybe_id) = self {
            maybe_id.replace(ids.into_iter().next().unwrap());

        // Otherwise, add our ids to our many
        } else if let Self::Many(existing_ids) = self {
            existing_ids.extend(ids);
        }

        Ok(())
    }

    /// Removes the provided ids from the edge value, failing if the ids would
    /// exceed the minimum allowed by the edge with current ids possibly
    /// removed
    pub fn remove_ids(
        &mut self,
        into_ids: impl IntoIterator<Item = usize>,
    ) -> Result<(), EdgeValueMutationError> {
        let ids = into_ids.into_iter().collect::<HashSet<usize>>();

        // Fails if we are not allowed to remove our id and we're given
        // some selection that would cause that issue
        if let Self::One(id) = self {
            if ids.contains(&id) {
                return Err(EdgeValueMutationError::InvalidatesEdge {
                    r#type: self.to_type(),
                });
            }
        }

        // Remove the id from our optional id if it is in our selection
        if let Self::MaybeOne(maybe_id) = self {
            if ids.contains(&maybe_id.unwrap()) {
                maybe_id.take();
            }
        // Remove all ids provided from our many
        } else if let Self::Many(existing_ids) = self {
            existing_ids.retain(|id| !ids.contains(id));
        }

        Ok(())
    }

    #[inline]
    fn id_count(&self) -> usize {
        match self {
            Self::MaybeOne(None) => 0,
            Self::MaybeOne(Some(_)) | Self::One(_) => 1,
            Self::Many(ids) => ids.len(),
        }
    }

    #[inline]
    fn max_ids_allowed(&self) -> usize {
        match self {
            Self::MaybeOne(_) | Self::One(_) => 1,
            Self::Many(_) => usize::MAX,
        }
    }

    #[inline]
    fn min_ids_allowed(&self) -> usize {
        match self {
            Self::MaybeOne(_) | Self::Many(_) => 0,
            Self::One(_) => 1,
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

    /// Returns true if this definition indicates that the edge should be
    /// involved in a shallow deletion if the ent is deleted
    pub fn should_shallow_delete(&self) -> bool {
        self.attributes
            .contains(&EdgeDefinitionAttribute::ShallowDelete)
    }

    /// Returns true if this definition indicates that the edge should be
    /// involved in a deep deletion if the ent is deleted
    pub fn should_deep_delete(&self) -> bool {
        self.attributes
            .contains(&EdgeDefinitionAttribute::DeepDelete)
    }
}
