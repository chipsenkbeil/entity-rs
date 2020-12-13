use crate::Id;
use derive_more::{From, TryInto};
use std::collections::HashSet;
use strum::{Display, EnumDiscriminants, EnumString};

/// Represents a definition of an edge, which is comprised of its name, type
/// of edge value, and the edge's deletion policy
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeDefinition {
    pub(super) name: String,
    r#type: EdgeValueType,
    deletion_policy: EdgeDeletionPolicy,
}

impl EdgeDefinition {
    /// Creates a new definition for an edge with the given name, type of value,
    /// and deletion policy of nothing
    pub fn new<N: Into<String>, T: Into<EdgeValueType>>(name: N, r#type: T) -> Self {
        Self::new_with_deletion_policy(name, r#type, EdgeDeletionPolicy::default())
    }

    /// Creates a new definition for an edge with the given name, type of value,
    /// and deletion policy
    pub fn new_with_deletion_policy<N: Into<String>, T: Into<EdgeValueType>>(
        name: N,
        r#type: T,
        deletion_policy: EdgeDeletionPolicy,
    ) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            deletion_policy,
        }
    }

    /// The name of the edge tied to the definition
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The type of value of the edge tied to the definition
    #[inline]
    pub fn r#type(&self) -> &EdgeValueType {
        &self.r#type
    }

    /// Returns the deletion policy of the edge tied to the definition
    #[inline]
    pub fn deletion_policy(&self) -> EdgeDeletionPolicy {
        self.deletion_policy
    }

    /// Returns true if the deletion policy is nothing
    #[inline]
    pub fn has_no_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::Nothing)
    }

    /// Returns true if the deletion policy is shallow
    #[inline]
    pub fn has_shallow_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::ShallowDelete)
    }

    /// Returns true if the deletion policy is deep
    #[inline]
    pub fn has_deep_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::DeepDelete)
    }
}

impl From<Edge> for EdgeDefinition {
    fn from(edge: Edge) -> Self {
        Self::new_with_deletion_policy(edge.name, edge.value, edge.deletion_policy)
    }
}

impl<'a> From<&'a Edge> for EdgeDefinition {
    fn from(edge: &'a Edge) -> Self {
        Self::new_with_deletion_policy(edge.name(), edge.value(), edge.deletion_policy())
    }
}

/// Represents a edge from an ent to one or more other ents
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Edge {
    name: String,
    value: EdgeValue,
    deletion_policy: EdgeDeletionPolicy,
}

impl Edge {
    /// Creates a new edge with the given name, value, and deletion policy
    /// of nothing
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Edge, EdgeDeletionPolicy};
    ///
    /// let edge = Edge::new("edge1", 999);
    /// assert_eq!(edge.name(), "edge1");
    /// assert_eq!(edge.to_ids(), vec![999]);
    /// assert_eq!(edge.deletion_policy(), EdgeDeletionPolicy::Nothing);
    /// ```
    pub fn new<N: Into<String>, V: Into<EdgeValue>>(name: N, value: V) -> Self {
        Self::new_with_deletion_policy(name, value, EdgeDeletionPolicy::default())
    }

    /// Creates a new edge with the given name, value, and deletion policy
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Edge, EdgeDeletionPolicy};
    ///
    /// let edge = Edge::new_with_deletion_policy(
    ///     "edge1",
    ///     999,
    ///     EdgeDeletionPolicy::DeepDelete,
    /// );
    /// assert_eq!(edge.name(), "edge1");
    /// assert_eq!(edge.to_ids(), vec![999]);
    /// assert_eq!(edge.deletion_policy(), EdgeDeletionPolicy::DeepDelete);
    /// ```
    pub fn new_with_deletion_policy<N: Into<String>, V: Into<EdgeValue>>(
        name: N,
        value: V,
        deletion_policy: EdgeDeletionPolicy,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            deletion_policy,
        }
    }

    /// The name of the edge
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value of the edge
    #[inline]
    pub fn value(&self) -> &EdgeValue {
        &self.value
    }

    /// The mutable value of the edge
    #[inline]
    pub fn value_mut(&mut self) -> &mut EdgeValue {
        &mut self.value
    }

    /// Converts edge into its value
    #[inline]
    pub fn into_value(self) -> EdgeValue {
        self.value
    }

    /// Converts to the ids of the ents referenced by this edge
    pub fn to_ids(&self) -> Vec<Id> {
        self.value.to_ids()
    }

    /// Converts to the edge's value type
    pub fn to_type(&self) -> EdgeValueType {
        self.value().into()
    }

    /// Returns the policy to perform for this edge when its ent is deleted
    #[inline]
    pub fn deletion_policy(&self) -> EdgeDeletionPolicy {
        self.deletion_policy
    }

    /// Returns true if the deletion policy is nothing
    #[inline]
    pub fn has_no_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::Nothing)
    }

    /// Returns true if the deletion policy is shallow
    #[inline]
    pub fn has_shallow_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::ShallowDelete)
    }

    /// Returns true if the deletion policy is deep
    #[inline]
    pub fn has_deep_deletion_policy(&self) -> bool {
        matches!(self.deletion_policy(), EdgeDeletionPolicy::DeepDelete)
    }
}

/// Represents the policy to apply to an edge when its ent is deleted
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeDeletionPolicy {
    /// When this ent instance is deleted, nothing else will be done
    Nothing,

    /// When this ent instance is deleted, delete the reverse edge connections
    /// of all ents connected by this edge
    ShallowDelete,

    /// When this ent instance is deleted, fully delete all ents connected
    /// by this edge
    DeepDelete,
}

impl Default for EdgeDeletionPolicy {
    /// By default, the deletion policy does nothing
    fn default() -> Self {
        Self::Nothing
    }
}

/// Represents the value of an edge, which is some collection of ent ids
#[derive(Clone, Debug, From, PartialEq, Eq, EnumDiscriminants, TryInto)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(Display, EnumString))]
#[strum_discriminants(name(EdgeValueType), strum(serialize_all = "snake_case"))]
#[cfg_attr(
    feature = "serde-1",
    strum_discriminants(derive(serde::Serialize, serde::Deserialize))
)]
pub enum EdgeValue {
    /// Edge can potentially have one outward connection
    MaybeOne(Option<Id>),
    /// Edge can have exactly one outward connection
    One(Id),
    /// Edge can have many outward connections
    Many(Vec<Id>),
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
    ///
    /// ## Examples
    ///
    /// For an optional edge, this can produce a vec of size 0 or 1:
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let v = EdgeValue::MaybeOne(None);
    /// assert_eq!(v.to_ids(), vec![]);
    ///
    /// let v = EdgeValue::MaybeOne(Some(999));
    /// assert_eq!(v.to_ids(), vec![999]);
    /// ```
    ///
    /// For a guaranteed edge of 1, this will always produce a vec of size 1:
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let v = EdgeValue::One(999);
    /// assert_eq!(v.to_ids(), vec![999]);
    /// ```
    ///
    /// For an edge of many ids, this will produce a vec of equal size:
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let v = EdgeValue::Many(vec![1, 2, 3, 4]);
    /// assert_eq!(v.to_ids(), vec![1, 2, 3, 4]);
    /// ```
    pub fn to_ids(&self) -> Vec<Id> {
        match self {
            Self::MaybeOne(x) => x.iter().copied().collect(),
            Self::One(x) => vec![*x],
            Self::Many(x) => x.clone(),
        }
    }

    /// Converts the value to its associated type
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{EdgeValue, EdgeValueType};
    ///
    /// let v = EdgeValue::MaybeOne(None);
    /// assert_eq!(v.to_type(), EdgeValueType::MaybeOne);
    ///
    /// let v = EdgeValue::One(999);
    /// assert_eq!(v.to_type(), EdgeValueType::One);
    ///
    /// let v = EdgeValue::Many(vec![1, 2, 3]);
    /// assert_eq!(v.to_type(), EdgeValueType::Many);
    /// ```
    pub fn to_type(&self) -> EdgeValueType {
        self.into()
    }

    /// Adds the provided ids to the edge value, failing if the ids would
    /// exceed the maximum allowed by the edge with current ids included
    ///
    /// ## Examples
    ///
    /// If edge is an optional, single value, this will fail if the edge
    /// already has a value or is provided more than one id. Otherwise,
    /// it will succeed.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::MaybeOne(Some(1));
    /// assert!(v.add_ids(vec![2]).is_err());
    /// assert!(v.add_ids(vec![]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(Some(1)));
    ///
    /// let mut v = EdgeValue::MaybeOne(None);
    /// assert!(v.add_ids(vec![2, 3]).is_err());
    /// assert_eq!(v, EdgeValue::MaybeOne(None));
    ///
    /// let mut v = EdgeValue::MaybeOne(None);
    /// assert!(v.add_ids(vec![]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(None));
    ///
    /// let mut v = EdgeValue::MaybeOne(None);
    /// assert!(v.add_ids(vec![1]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(Some(1)));
    /// ```
    ///
    /// If an edge is exactly one value, this will fail unless an empty
    /// list of ids is given as we cannot add any more edge ids.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::One(999);
    /// assert!(v.add_ids(vec![1]).is_err());
    /// assert_eq!(v, EdgeValue::One(999));
    ///
    /// let mut v = EdgeValue::One(999);
    /// assert!(v.add_ids(vec![]).is_ok());
    /// assert_eq!(v, EdgeValue::One(999));
    /// ```
    ///
    /// If an edge can have many ids, this will succeed and append those
    /// ids to the end of the list.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::Many(vec![]);
    /// assert!(v.add_ids(vec![1, 2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::Many(vec![1, 2, 3]));
    ///
    /// let mut v = EdgeValue::Many(vec![1, 2, 3]);
    /// assert!(v.add_ids(vec![4, 5, 6]).is_ok());
    /// assert_eq!(v, EdgeValue::Many(vec![1, 2, 3, 4, 5, 6]));
    /// ```
    pub fn add_ids(
        &mut self,
        into_ids: impl IntoIterator<Item = Id>,
    ) -> Result<(), EdgeValueMutationError> {
        let mut ids = into_ids.into_iter().collect::<Vec<Id>>();
        ids.sort_unstable();
        ids.dedup();

        // If no ids to add, will always succeed and do nothing
        if ids.is_empty() {
            return Ok(());
        }

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
    ///
    /// ## Examples
    ///
    /// If edge is an optional, single value, this will never fail as this
    /// can either result in the value retaining its id or becoming none.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::MaybeOne(Some(1));
    /// assert!(v.remove_ids(vec![2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(Some(1)));
    ///
    /// let mut v = EdgeValue::MaybeOne(Some(1));
    /// assert!(v.remove_ids(vec![1, 2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(None));
    ///
    /// let mut v = EdgeValue::MaybeOne(None);
    /// assert!(v.remove_ids(vec![2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(None));
    ///
    /// let mut v = EdgeValue::MaybeOne(None);
    /// assert!(v.remove_ids(vec![]).is_ok());
    /// assert_eq!(v, EdgeValue::MaybeOne(None));
    /// ```
    ///
    /// If an edge is exactly one value, this will fail if it would cause
    /// the value to lose its id.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::One(999);
    /// assert!(v.remove_ids(vec![999]).is_err());
    /// assert_eq!(v, EdgeValue::One(999));
    ///
    /// let mut v = EdgeValue::One(999);
    /// assert!(v.remove_ids(vec![1, 2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::One(999));
    ///
    /// let mut v = EdgeValue::One(999);
    /// assert!(v.remove_ids(vec![]).is_ok());
    /// assert_eq!(v, EdgeValue::One(999));
    /// ```
    ///
    /// If an edge can have many ids, this will succeed and remove all ids
    /// found within the value.
    ///
    /// ```
    /// use entity::EdgeValue;
    ///
    /// let mut v = EdgeValue::Many(vec![]);
    /// assert!(v.remove_ids(vec![1, 2, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::Many(vec![]));
    ///
    /// let mut v = EdgeValue::Many(vec![1, 2, 3]);
    /// assert!(v.remove_ids(vec![4, 5, 6]).is_ok());
    /// assert_eq!(v, EdgeValue::Many(vec![1, 2, 3]));
    ///
    /// let mut v = EdgeValue::Many(vec![1, 2, 3]);
    /// assert!(v.remove_ids(vec![1, 3]).is_ok());
    /// assert_eq!(v, EdgeValue::Many(vec![2]));
    /// ```
    pub fn remove_ids(
        &mut self,
        into_ids: impl IntoIterator<Item = Id>,
    ) -> Result<(), EdgeValueMutationError> {
        let ids = into_ids.into_iter().collect::<HashSet<Id>>();

        // If no ids to remove, will always succeed and do nothing
        if ids.is_empty() {
            return Ok(());
        }

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
            if maybe_id.is_some() && ids.contains(&maybe_id.unwrap()) {
                maybe_id.take();
            }
        // Remove all ids provided from our many
        } else if let Self::Many(existing_ids) = self {
            existing_ids.retain(|id| !ids.contains(id));
        }

        Ok(())
    }

    /// Returns the total ids contained within this value
    #[inline]
    fn id_count(&self) -> usize {
        match self {
            Self::MaybeOne(None) => 0,
            Self::MaybeOne(Some(_)) | Self::One(_) => 1,
            Self::Many(ids) => ids.len(),
        }
    }

    /// Returns the maximum ids possible to contain within this value
    #[inline]
    fn max_ids_allowed(&self) -> usize {
        match self {
            Self::MaybeOne(_) | Self::One(_) => 1,
            Self::Many(_) => usize::MAX,
        }
    }
}
