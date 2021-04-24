use entity_async_graphql::gql_ent;

#[gql_ent]
struct TestEnt1 {
    field1: bool,
}

#[gql_ent]
struct TestEnt2 {
    field1: usize,
    field2: f32,
}

#[test]
fn adds_simple_ent_attribute() {
    #[gql_ent]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let ent = TestEntEnum::One(TestEnt1 {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    });

    use entity::Ent;
    assert_eq!(ent.id(), 123);
    assert!(!ent.is_connected());
    assert_eq!(ent.created(), 456);
    assert_eq!(ent.last_updated(), 789);
}

#[test]
fn fills_in_derive_async_graphql_union_when_missing() {
    #[gql_ent]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    struct RootQuery;

    #[async_graphql::Object]
    impl RootQuery {
        async fn ent(&self) -> TestEntEnum {
            TestEntEnum::One(TestEnt1 {
                id: 123,
                database: entity::WeakDatabaseRc::new(),
                created: 456,
                last_updated: 789,
                field1: true,
            })
        }
    }

    let schema = async_graphql::Schema::new(
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    );

    let res = futures::executor::block_on(schema.execute("{ ent { ... on TestEnt1 { field1 } } }"));
    let data = res.data.into_json().unwrap();
    assert_eq!(data, serde_json::json!({ "ent": { "field1": true } }));
}
