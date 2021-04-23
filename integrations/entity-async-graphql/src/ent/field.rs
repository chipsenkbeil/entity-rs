use super::GqlValue;
use async_graphql::Object;
use derive_more::{From, Into};
use entity::Field;

/// Represents a wrapper around a `Field` to expose as graphql
#[derive(From, Into)]
pub struct GqlField(Field);

#[Object]
impl GqlField {
    #[graphql(name = "name")]
    async fn gql_name(&self) -> &str {
        self.0.name()
    }

    #[graphql(name = "value")]
    async fn gql_value(&self) -> GqlValue {
        GqlValue::from(self.0.value().clone())
    }

    #[graphql(name = "indexed")]
    async fn gql_indexed(&self) -> bool {
        self.0.is_indexed()
    }

    #[graphql(name = "immutable")]
    async fn gql_immutable(&self) -> bool {
        self.0.is_immutable()
    }
}
