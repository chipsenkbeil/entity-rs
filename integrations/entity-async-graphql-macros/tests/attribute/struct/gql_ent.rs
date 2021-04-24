use entity_async_graphql::{gql_ent, EntFilter, EntObject};

#[test]
fn adds_simple_ent_attribute() {
    #[gql_ent]
    struct TestEnt {
        field1: bool,
    }

    let ent = TestEnt {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    };

    use entity::Ent;
    assert_eq!(ent.id(), 123);
    assert!(!ent.is_connected());
    assert_eq!(ent.created(), 456);
    assert_eq!(ent.last_updated(), 789);
}

#[test]
fn fills_in_derive_ent_object_when_missing() {
    #[gql_ent]
    #[derive(EntFilter)]
    struct TestEnt {
        field1: bool,
    }

    struct RootQuery;

    #[async_graphql::Object]
    impl RootQuery {
        async fn ent(&self) -> TestEnt {
            TestEnt {
                id: 123,
                database: entity::WeakDatabaseRc::new(),
                created: 456,
                last_updated: 789,
                field1: true,
            }
        }
    }

    let schema = async_graphql::Schema::new(
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    );

    let res = futures::executor::block_on(schema.execute("{ ent { field1 } }"));
    let data = res.data.into_json().unwrap();
    assert_eq!(data, serde_json::json!({ "ent": { "field1": true } }));
}

#[test]
fn fills_in_derive_ent_filter_when_missing() {
    #[gql_ent]
    #[derive(EntObject)]
    struct TestEnt {
        field1: bool,
    }

    let _ent = TestEnt {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    };

    // For now, just verify that the struct type exists
    let _filter = std::any::type_name::<GqlTestEntFilter>();
}
