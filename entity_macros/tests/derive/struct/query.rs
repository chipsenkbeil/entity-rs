use entity::{TypedPredicate as P, *};

#[test]
fn produces_method_to_filter_by_id() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_id(P::equals(2))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}

#[test]
fn produces_methods_to_filter_by_created_timestamp() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 100,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 200,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 3,
            database: WeakDatabaseRc::new(),
            created: 300,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_created(P::less_than(200))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn produces_methods_to_filter_by_last_updated_timestamp() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    // Sleeping to make sure that our millisecond timestamp progresses
    // before adding the next entry
    std::thread::sleep(std::time::Duration::from_millis(10));

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    // Sleeping to make sure that our millisecond timestamp progresses
    // before adding the next entry
    std::thread::sleep(std::time::Duration::from_millis(10));

    database
        .insert(Box::from(TestEnt {
            id: 3,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    // NOTE: Databases update the last_updated field upon insert, so we need
    // to pull the ents back out to see what the values are
    let ent2_last_updated = database.get(2).unwrap().unwrap().last_updated();

    let results: Vec<Id> = TestEntQuery::default()
        .where_last_updated(P::less_than(ent2_last_updated))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn produces_method_to_filter_by_field() {
    #[derive(Clone, Ent, EntQuery, EntType)]
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
        value: u32,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_value(P::equals(100))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn supports_generic_fields() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt<T>
    where
        T: ValueLike + Clone + Send + Sync + 'static,
    {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        value: T,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_value(P::equals(200))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}

#[test]
fn produces_method_to_filter_by_ents_connected_by_edge() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        other: Id,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 1,
        }))
        .expect("Failed to insert a test ent");

    use entity::DatabaseExt;
    let ent2: TestEnt = database.get_typed(2).unwrap().unwrap();

    let results: Vec<Id> = TestEntQuery::query_from_other(&ent2)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn produces_method_to_yield_edge_ents() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt1 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt2"))]
        other: Id,
    }

    #[derive(Clone, Ent, EntQuery, EntType)]
    struct TestEnt2 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt1"))]
        other: Id,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt2 {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 1,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEnt1Query::default()
        .query_other()
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}

#[test]
fn supports_providing_an_explicit_query_type_on_edges() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    pub struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        // Specify a generic query type instead of using TestEntQuery by default
        #[ent(edge(type = "TestEnt", query_type = "Query"))]
        other: Id,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            other: 1,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_id(P::equals(1))
        .query_other()
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(|boxed_ent| boxed_ent.id())
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}

mod type_path {
    use ent1::{TestEnt1, TestEnt1Query as QueryForTestEnt1};
    use ent2::{TestEnt2, TestEnt2Query as QueryForTestEnt2};
    use entity::*;

    mod ent1 {
        use entity::*;

        #[derive(Clone, Ent, EntQuery, EntType)]
        pub struct TestEnt1 {
            #[ent(id)]
            pub id: Id,

            #[ent(database)]
            pub database: WeakDatabaseRc,

            #[ent(created)]
            pub created: u64,

            #[ent(last_updated)]
            pub last_updated: u64,

            #[ent(edge(type = "super::TestEnt2", query_type = "super::QueryForTestEnt2"))]
            pub other: Id,
        }
    }

    mod ent2 {
        use entity::*;

        #[derive(Clone, Ent, EntQuery, EntType)]
        pub struct TestEnt2 {
            #[ent(id)]
            pub id: Id,

            #[ent(database)]
            pub database: WeakDatabaseRc,

            #[ent(created)]
            pub created: u64,

            #[ent(last_updated)]
            pub last_updated: u64,

            #[ent(edge(type = "super::TestEnt1", query_type = "super::QueryForTestEnt1"))]
            pub other: Id,
        }
    }

    #[test]
    fn supports_providing_an_explicit_query_type_path_on_edges() {
        let database = InmemoryDatabase::default();

        database
            .insert(Box::from(TestEnt1 {
                id: 1,
                database: WeakDatabaseRc::new(),
                created: 0,
                last_updated: 0,
                other: 2,
            }))
            .expect("Failed to insert a test ent");

        database
            .insert(Box::from(TestEnt2 {
                id: 2,
                database: WeakDatabaseRc::new(),
                created: 0,
                last_updated: 0,
                other: 1,
            }))
            .expect("Failed to insert a test ent");

        let results: Vec<Id> = QueryForTestEnt1::default()
            .query_other()
            .execute(&database)
            .expect("Failed to query for ents")
            .iter()
            .map(Ent::id)
            .collect();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&2));
    }
}
