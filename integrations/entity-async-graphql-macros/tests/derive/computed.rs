use entity::{Database, DatabaseExt, DatabaseRc, Ent, EntTypedFields, Id, WeakDatabaseRc};
use entity_async_graphql_macros::{EntFilter, EntObject};
use entity_inmemory::InmemoryDatabase;

const TEST_ENT_TYPE: &str = concat!(module_path!(), "::TestEnt");

// NOTE: We need EntTypedEdges for now, but if the macro is updated to not
//       require it then we can remove that constraint
#[derive(Clone, Ent, EntTypedFields, EntObject, EntFilter)]
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
    a: String,

    #[ent(field)]
    b: u32,

    #[ent(field(computed = "format!(\"{} : {}\", self.a, self.b)"))]
    c: Option<String>,
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
            a: "one".to_string(),
            b: 123,
            c: None,
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
            a: "two".to_string(),
            b: 456,
            c: None,
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
fn supports_retrieving_computed_ent_fields() {
    let schema = make_schema();
    let result = execute(&schema, "{ find(id: 1) { c } }");
    assert_eq!(result, serde_json::json!({ "find": [{"c": "one : 123"}] }));
}

#[test]
fn supports_filtering_by_computed_ent_fields() {
    let schema = make_schema();
    let result = execute(
        &schema,
        "{ find(filter: { c: { text_ends_with: \"123\" }}) { c } }",
    );
    assert_eq!(result, serde_json::json!({ "find": [{"c": "one : 123"}] }));
}
