use async_graphql::InputObject;
use entity::{Filter, Predicate, Query};

mod float;
pub use float::*;

mod primitive;
pub use primitive::*;

/// Represents a wrapper around an ent query [`Filter`] that exposes a GraphQL API.
#[derive(Clone, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GqlEntFilter {
    /// Filter by ent's id
    id: Option<GqlPredicate_Id>,

    /// Filter by ent's type
    #[graphql(name = "type")]
    r#type: Option<GqlPredicate_String>,

    /// Filter by ent's creation timestamp
    created: Option<GqlPredicate_u64>,

    /// Filter by ent's last updated timestamp
    last_updated: Option<GqlPredicate_u64>,

    /// Filter by ent's fields
    fields: Option<Vec<GqlEntFieldFilter>>,

    /// Filter by ent's edges
    edges: Option<Vec<GqlEntEdgeFilter>>,
}

impl From<GqlEntFilter> for Query {
    /// Converts [`GqlEntFilter`] to [`Query`] by chaining together all conditions
    /// contained within the GraphQL filter in this order:
    ///
    /// 1. id
    /// 2. type
    /// 3. created
    /// 4. last_updated
    /// 5. fields
    /// 6. edges
    fn from(x: GqlEntFilter) -> Self {
        let mut query = Query::default();

        if let Some(pred) = x.id {
            query.add_filter(Filter::where_id(Predicate::from(pred)));
        }

        if let Some(pred) = x.r#type {
            query.add_filter(Filter::where_type(Predicate::from(pred)));
        }

        if let Some(pred) = x.created {
            query.add_filter(Filter::where_created(Predicate::from(pred)));
        }

        if let Some(pred) = x.last_updated {
            query.add_filter(Filter::where_last_updated(Predicate::from(pred)));
        }

        if let Some(gql_filters) = x.fields {
            for f in gql_filters {
                query.add_filter(Filter::where_field(f.name, f.predicate));
            }
        }

        if let Some(gql_filters) = x.edges {
            for f in gql_filters {
                let edge_query = Query::from(f.filter.as_ref().clone());
                for edge_filter in edge_query {
                    query.add_filter(Filter::where_edge(&f.name, edge_filter));
                }
            }
        }

        query
    }
}

#[derive(Clone, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GqlEntFieldFilter {
    name: String,
    predicate: GqlPredicate_Value,
}

#[derive(Clone, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GqlEntEdgeFilter {
    name: String,
    filter: Box<GqlEntFilter>,
}
