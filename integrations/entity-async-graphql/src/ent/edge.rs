use async_graphql::{Enum, Object};
use derive_more::{From, Into};
use entity::{Edge, EdgeDeletionPolicy, EdgeValue, EdgeValueType, Id};

/// Represents a wrapper around an `Edge` to expose as graphql
#[derive(From, Into)]
pub struct GqlEdge(Edge);

#[Object]
impl GqlEdge {
    #[graphql(name = "name")]
    async fn gql_name(&self) -> &str {
        self.0.name()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> GqlEdgeValueType {
        GqlEdgeValueType::from(self.0.to_type())
    }

    #[graphql(name = "value")]
    async fn gql_value(&self) -> GqlEdgeValue {
        GqlEdgeValue::from(self.0.value().clone())
    }

    #[graphql(name = "ids")]
    async fn gql_ids(&self) -> Vec<Id> {
        self.0.to_ids()
    }

    #[graphql(name = "deletion_policy")]
    async fn gql_deletion_policy(&self) -> GqlEdgeDeletionPolicy {
        GqlEdgeDeletionPolicy::from(self.0.deletion_policy())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Enum)]
#[graphql(remote = "EdgeDeletionPolicy")]
pub enum GqlEdgeDeletionPolicy {
    /// When this ent instance is deleted, nothing else will be done
    Nothing,

    /// When this ent instance is deleted, delete the reverse edge connections
    /// of all ents connected by this edge
    ShallowDelete,

    /// When this ent instance is deleted, fully delete all ents connected
    /// by this edge
    DeepDelete,
}

/// Represents a wrapper around an `EdgeValue` to expose as graphql
#[derive(From, Into)]
pub struct GqlEdgeValue(EdgeValue);

#[Object]
impl GqlEdgeValue {
    #[graphql(name = "ids")]
    async fn gql_ids(&self) -> Vec<Id> {
        self.0.to_ids()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> GqlEdgeValueType {
        GqlEdgeValueType::from(self.0.to_type())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Enum)]
#[graphql(remote = "EdgeValueType")]
pub enum GqlEdgeValueType {
    /// Edge can potentially have one outward connection
    MaybeOne,
    /// Edge can have exactly one outward connection
    One,
    /// Edge can have many outward connections
    Many,
}
