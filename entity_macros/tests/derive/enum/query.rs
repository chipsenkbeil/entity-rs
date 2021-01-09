use derivative::Derivative;
use entity::{TypedPredicate as P, *};
use std::convert::TryFrom;

#[derive(Clone, Derivative, Ent, EntType, EntQuery)]
#[derivative(Debug)]
struct TestEnt1 {
    #[ent(id)]
    id: Id,

    #[derivative(Debug = "ignore")]
    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    field1: usize,

    #[ent(edge(type = "TestEnt2"))]
    other: Id,
}

#[derive(Clone, Derivative, Ent, EntType, EntQuery)]
#[derivative(Debug)]
struct TestEnt2 {
    #[ent(id)]
    id: Id,

    #[derivative(Debug = "ignore")]
    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    field1: usize,

    #[ent(field)]
    field2: String,

    #[ent(edge(type = "TestEnt1"))]
    maybe_other: Option<Id>,

    #[ent(edge(type = "TestEnt2"))]
    dups: Vec<Id>,
}

#[derive(Clone, Debug, Ent, EntQuery, EntType, EntWrapper)]
enum TestEnt {
    One(TestEnt1),
    Two(TestEnt2),
}

#[test]
fn produces_method_to_filter_by_id() {
    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 999,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt2 {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 1000,
            field2: String::from("test"),
            maybe_other: Some(1),
            dups: Vec::new(),
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
    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 100,
            last_updated: 0,
            field1: 999,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt2 {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 200,
            last_updated: 0,
            field1: 1000,
            field2: String::from("test"),
            maybe_other: Some(1),
            dups: Vec::new(),
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
    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 999,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    // Sleeping to make sure that our millisecond timestamp progresses
    // before adding the next entry
    std::thread::sleep(std::time::Duration::from_millis(10));

    database
        .insert(Box::from(TestEnt2 {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 1000,
            field2: String::from("test"),
            maybe_other: Some(1),
            dups: Vec::new(),
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
    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 999,
            other: 2,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(TestEnt2 {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            field1: 1000,
            field2: String::from("test"),
            maybe_other: Some(1),
            dups: Vec::new(),
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = TestEntQuery::default()
        .where_field("field1", P::equals(1000).into())
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&2));
}

#[test]
fn supports_generic_fields() {
    #[derive(Clone, Ent, EntQuery, EntType)]
    struct GenericTestEnt<T>
    where
        T: TryFrom<Value, Error = &'static str> + Into<Value> + Clone + Send + Sync + 'static,
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

    #[derive(Clone, Ent, EntQuery, EntType, EntWrapper)]
    enum GenericTestEntEnum<T>
    where
        T: TryFrom<Value, Error = &'static str> + Into<Value> + Clone + Send + Sync + 'static,
    {
        Choice(GenericTestEnt<T>),
    }

    let database = InmemoryDatabase::default();

    database
        .insert(Box::from(GenericTestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 100usize,
        }))
        .expect("Failed to insert a test ent");

    database
        .insert(Box::from(GenericTestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 0,
            last_updated: 0,
            value: 200usize,
        }))
        .expect("Failed to insert a test ent");

    let results: Vec<Id> = GenericTestEntEnumQuery::<usize>::default()
        .where_field("value", P::equals(200).into())
        .execute(&database)
        .expect("Failed to query for ents")
        .iter()
        .map(Ent::id)
        .collect();
    assert_eq!(results.len(), 1, "Unexpected total results");
    assert!(results.contains(&2));
}
