use entity::*;

const TEST_ENT_TYPE: &str = concat!(module_path!(), "::TestEnt");

// NOTE: We need EntTypedEdges for now, but if the macro is updated to not
//       require it then we can remove that constraint
#[derive(Clone, Ent, EntTypedEdges, AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    my_field: String,

    #[ent(edge(type = "TestEnt"))]
    my_maybe_edge: Option<Id>,

    #[ent(edge(type = "TestEnt"))]
    my_edge: Id,

    #[ent(edge(type = "TestEnt"))]
    my_many_edges: Vec<Id>,
}

struct RootQuery;

#[async_graphql::Object]
impl RootQuery {
    async fn find(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: Option<Id>,
        filter: Option<GqlTestEntFilter>,
    ) -> async_graphql::Result<Vec<TestEnt>> {
        let db = ctx.data::<DatabaseRc>()?;

        if let Some(id) = id {
            match db.get_all_typed::<TestEnt>(vec![id]) {
                Ok(mut ents) => {
                    for ent in ents.iter_mut() {
                        ent.connect(DatabaseRc::downgrade(&db));
                    }
                    Ok(ents)
                }
                Err(x) => Err(async_graphql::Error::new(x.to_string())),
            }
        } else if let Some(filter) = filter {
            match db.find_all_typed::<TestEnt>(filter.into()) {
                Ok(mut ents) => {
                    for ent in ents.iter_mut() {
                        ent.connect(DatabaseRc::downgrade(&db));
                    }
                    Ok(ents)
                }
                Err(x) => Err(async_graphql::Error::new(x.to_string())),
            }
        } else {
            Err(async_graphql::Error::new("Must provide one argument"))
        }
    }
}

#[inline]
fn make_db() -> DatabaseRc {
    let db = InmemoryDatabase::default();
    let _ = db
        .insert(Box::new(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 111,
            last_updated: 0,
            my_field: "one".to_string(),
            my_maybe_edge: Some(2),
            my_edge: 2,
            my_many_edges: vec![2],
        }))
        .unwrap();

    // Delay next ent creation so we can ensure a different last_updated time
    std::thread::sleep(::std::time::Duration::from_millis(10));

    let _ = db
        .insert(Box::new(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 222,
            last_updated: 0,
            my_field: "two".to_string(),
            my_maybe_edge: Some(1),
            my_edge: 1,
            my_many_edges: vec![1],
        }))
        .unwrap();

    DatabaseRc::new(Box::new(db))
}

#[inline]
fn make_schema_and_db() -> (
    async_graphql::Schema<
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    >,
    DatabaseRc,
) {
    let db = make_db();
    let schema = async_graphql::Schema::build(
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(DatabaseRc::clone(&db))
    .finish();
    (schema, db)
}

#[inline]
fn make_schema(
) -> async_graphql::Schema<RootQuery, async_graphql::EmptyMutation, async_graphql::EmptySubscription>
{
    let (schema, _) = make_schema_and_db();
    schema
}

#[inline]
fn execute(
    schema: &async_graphql::Schema<
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    >,
    input: &str,
) -> serde_json::Value {
    let input = input.trim();
    let res = futures::executor::block_on(schema.execute(input));
    let data = res.data.into_json().unwrap();
    assert_ne!(
        data,
        serde_json::Value::Null,
        "{}",
        serde_json::to_string_pretty(&res.errors).unwrap()
    );
    data
}

#[test]
fn supports_id_field() {
    let schema = make_schema();
    let result = execute(&schema, "{ find(id: 1) { id } }");
    assert_eq!(result, serde_json::json!({ "find": [{"id": 1}] }));
}

#[test]
fn supports_type_field() {
    let schema = make_schema();
    let result = execute(&schema, "{ find(id: 1) { type } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"type": TEST_ENT_TYPE}] })
    );
}

#[test]
fn supports_created_field() {
    let (schema, db) = make_schema_and_db();
    let created = db.get(1).unwrap().unwrap().created();

    let result = execute(&schema, "{ find(id: 1) { created } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"created": created}] })
    );
}

#[test]
fn supports_last_updated_field() {
    let (schema, db) = make_schema_and_db();
    let last_updated = db.get(1).unwrap().unwrap().last_updated();

    let result = execute(&schema, "{ find(id: 1) { last_updated } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"last_updated": last_updated}] })
    );
}

#[test]
fn supports_ent_fields() {
    let schema = make_schema();
    let result = execute(&schema, "{ find(id: 1) { my_field } }");
    assert_eq!(result, serde_json::json!({ "find": [{"my_field": "one"}] }));
}

#[test]
fn supports_ent_edge_ids() {
    let schema = make_schema();

    let result = execute(&schema, "{ find(id: 1) { id_for_my_maybe_edge } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"id_for_my_maybe_edge": 2}] })
    );

    let result = execute(&schema, "{ find(id: 1) { id_for_my_edge } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"id_for_my_edge": 2}] })
    );

    let result = execute(&schema, "{ find(id: 1) { ids_for_my_many_edges } }");
    assert_eq!(
        result,
        serde_json::json!({ "find": [{"ids_for_my_many_edges": [2]}] })
    );
}

#[test]
fn supports_loading_ent_edges() {
    let schema = make_schema();

    let result = execute(
        &schema,
        "{ find(id: 1) { my_maybe_edge { id, my_field } } }",
    );
    assert_eq!(
        result,
        serde_json::json!({
            "find": [
                {"my_maybe_edge": {
                    "id": 2,
                    "my_field": "two"
                }}
            ]
        })
    );

    let result = execute(&schema, "{ find(id: 1) { my_edge { id, my_field } } }");
    assert_eq!(
        result,
        serde_json::json!({
            "find": [
                {"my_edge": {
                    "id": 2,
                    "my_field": "two"
                }}
            ]
        })
    );

    let result = execute(
        &schema,
        "{ find(id: 1) { my_many_edges { id, my_field } } }",
    );
    assert_eq!(
        result,
        serde_json::json!({
            "find": [
                {"my_many_edges": [{
                    "id": 2,
                    "my_field": "two"
                }]}
            ]
        })
    );
}

#[test]
fn supports_filtering_by_id() {
    let schema = make_schema();
    let result = execute(&schema, "{ find(filter: { id: { equals: 1 } }) { id } }");
    assert_eq!(result, serde_json::json!({ "find": [{"id": 1}] }));
}

#[test]
fn supports_filtering_by_created() {
    let schema = make_schema();
    let result = execute(
        &schema,
        "{ find(filter: { created: { equals: 111 } }) { id } }",
    );
    assert_eq!(result, serde_json::json!({ "find": [{"id": 1}] }));
}

#[test]
fn supports_filtering_by_last_updated() {
    let (schema, db) = make_schema_and_db();
    let last_updated = db.get(1).unwrap().unwrap().last_updated();

    let result = execute(
        &schema,
        &format!(
            "{{ find(filter: {{ last_updated: {{ not_equals: {} }} }}) {{ id }} }}",
            last_updated,
        ),
    );
    assert_eq!(result, serde_json::json!({ "find": [{"id": 2}] }));
}

#[test]
fn supports_filtering_by_ent_fields() {
    let schema = make_schema();
    let result = execute(
        &schema,
        "{ find(filter: { my_field: { text_ends_with: \"wo\" } }) { id } }",
    );
    assert_eq!(result, serde_json::json!({ "find": [{"id": 2}] }));
}

#[test]
fn supports_filtering_by_ent_edges() {
    let schema = make_schema();
    let result = execute(
        &schema,
        "{ find(filter: { my_edge: { id: { equals: 1 } } }) { id } }",
    );
    assert_eq!(result, serde_json::json!({ "find": [{"id": 2}] }));
}
