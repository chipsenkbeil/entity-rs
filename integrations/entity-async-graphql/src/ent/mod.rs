use async_graphql::{Error, Object, Result};
use derive_more::{From, Into};
use entity::{Ent, Id};

mod edge;
mod field;
mod value;

pub use edge::{GqlEdge, GqlEdgeDeletionPolicy, GqlEdgeValue, GqlEdgeValueType};
pub use field::GqlField;
pub use value::GqlValue;

/// Represents a wrapper around a `dyn Ent` to expose as graphql
#[derive(From, Into)]
pub struct GqlDynEnt(Box<dyn Ent>);

#[Object]
impl GqlDynEnt {
    #[graphql(name = "id")]
    async fn gql_id(&self) -> Id {
        self.0.id()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> &str {
        self.0.r#type()
    }

    #[graphql(name = "created")]
    async fn gql_created(&self) -> u64 {
        self.0.created()
    }

    #[graphql(name = "last_updated")]
    async fn gql_last_updated(&self) -> u64 {
        self.0.last_updated()
    }

    #[graphql(name = "field")]
    async fn gql_field(&self, name: String) -> Option<GqlValue> {
        self.0.field(&name).map(GqlValue::from)
    }

    #[graphql(name = "fields")]
    async fn gql_fields(&self) -> Vec<GqlField> {
        self.0.fields().into_iter().map(GqlField::from).collect()
    }

    #[graphql(name = "edge")]
    async fn gql_edge(&self, name: String) -> Option<GqlEdgeValue> {
        self.0.edge(&name).map(GqlEdgeValue::from)
    }

    #[graphql(name = "edges")]
    async fn gql_edges(&self) -> Vec<GqlEdge> {
        self.0.edges().into_iter().map(GqlEdge::from).collect()
    }

    #[graphql(name = "load_edge")]
    async fn gql_load_edge(&self, name: String) -> Result<Vec<Self>> {
        self.0
            .load_edge(&name)
            .map(|x| x.into_iter().map(Self::from).collect())
            .map_err(|x| Error::new(x.to_string()))
    }
}
