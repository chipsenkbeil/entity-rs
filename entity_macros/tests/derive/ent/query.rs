use entity::{Database, Ent, IEnt, Id, InmemoryDatabase, Value};
use std::convert::TryFrom;

#[test]
fn produces_method_to_filter_by_id() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    // Search by exact id
    let results: Vec<Id> = TestEntQuery::default()
        .with_id(2)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));

    // Search by any id of vec
    let results: Vec<Id> = TestEntQuery::default()
        .with_any_id(vec![1, 2])
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
}

#[test]
fn produces_methods_to_filter_by_created_timestamp() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: None,
            created: 100,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 200,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 3,
            database: None,
            created: 300,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .created_before(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));

    let results: Vec<Id> = TestEntQuery::default()
        .created_on_or_before(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));

    let results: Vec<Id> = TestEntQuery::default()
        .created_after(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&3));

    let results: Vec<Id> = TestEntQuery::default()
        .created_on_or_after(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&2));
    assert!(results.contains(&3));

    let results: Vec<Id> = TestEntQuery::default()
        .created_between(100, 300)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));

    let results: Vec<Id> = TestEntQuery::default()
        .created_on_or_between(100, 300)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 3);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
    assert!(results.contains(&3));
}

#[test]
fn produces_methods_to_filter_by_last_updated_timestamp() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: None,
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
            database: None,
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
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    // NOTE: Databases update the last_updated field upon insert, so we need
    // to pull the ents back out to see what the values are
    let ent1_last_updated = database.get(1).unwrap().unwrap().last_updated();
    let ent2_last_updated = database.get(2).unwrap().unwrap().last_updated();
    let ent3_last_updated = database.get(3).unwrap().unwrap().last_updated();

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_before(ent2_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_on_or_before(ent2_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_after(ent2_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&3));

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_on_or_after(ent2_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&2));
    assert!(results.contains(&3));

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_between(ent1_last_updated, ent3_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));

    let results: Vec<Id> = TestEntQuery::default()
        .last_updated_on_or_between(ent1_last_updated, ent3_last_updated)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 3);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
    assert!(results.contains(&3));
}

#[test]
fn produces_method_to_combine_two_typed_queries_with_logical_or() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .with_id(2)
        .or(TestEntQuery::default().with_id(1))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
}

#[test]
fn produces_method_to_combine_two_typed_queries_with_logical_xor() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt {
            id: 1,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .with_id(2)
        .xor(TestEntQuery::default().created_on_or_before(0))
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn produces_method_to_filter_by_field_equality() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

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
            database: None,
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .value_eq(100)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));
}

#[test]
fn produces_method_to_filter_by_field_greater_than() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

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
            database: None,
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .value_gt(100)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));

    let results: Vec<Id> = TestEntQuery::default()
        .value_gt(0)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
}

#[test]
fn produces_method_to_filter_by_field_less_than() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

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
            database: None,
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .value_lt(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&1));

    let results: Vec<Id> = TestEntQuery::default()
        .value_lt(300)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
}

#[test]
fn supports_generic_fields() {
    #[derive(Clone, Ent)]
    #[ent(query)]
    struct TestEnt<T>
    where
        T: TryFrom<Value, Error = &'static str> + Into<Value> + Clone + 'static,
    {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

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
            database: None,
            created: 0,
            last_updated: 0,
            value: 100,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt {
            id: 2,
            database: None,
            created: 0,
            last_updated: 0,
            value: 200,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .value_eq(200)
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(IEnt::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}
