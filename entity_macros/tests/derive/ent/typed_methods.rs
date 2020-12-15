use entity::{Database, DatabaseError, Ent, IEnt, Id, InmemoryDatabase, Value};
use std::convert::TryFrom;

#[test]
fn produces_getters_for_fields_that_returns_references() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
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
        my_field1: u32,

        #[ent(field)]
        my_field2: String,
    }

    let ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_field1: 123,
        my_field2: String::from("test"),
    };

    assert_eq!(ent.my_field1(), &123);
    assert_eq!(ent.my_field2(), "test");
}

#[test]
fn produces_setters_for_fields_marked_as_mutable() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field(mutable))]
        my_field1: u32,

        #[ent(field(mutable))]
        my_field2: String,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_field1: 123,
        my_field2: String::from("test"),
    };

    assert_eq!(ent.set_my_field1(1000), 123);
    assert_eq!(ent.my_field1, 1000);

    assert_eq!(
        ent.set_my_field2(String::from("something")),
        String::from("test")
    );
    assert_eq!(ent.my_field2, String::from("something"));
}

#[test]
fn produces_getters_for_edge_ids_that_returns_an_option_if_kind_is_maybe() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Option<Id>,
    }

    let ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: Some(123),
    };

    assert_eq!(ent.my_edge_id(), Some(123));
}

#[test]
fn produces_getters_for_edge_ids_that_returns_the_id_if_kind_is_one() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Id,
    }

    let ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: 123,
    };

    assert_eq!(ent.my_edge_id(), 123);
}

#[test]
fn produces_getters_for_edge_ids_that_returns_a_list_of_ids_if_kind_is_many() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Vec<Id>,
    }

    let ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: vec![123, 456],
    };

    assert_eq!(ent.my_edge_ids(), &[123, 456]);
}

#[test]
fn produces_setters_for_edge_ids_that_accepts_an_optional_id_if_kind_is_maybe() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Option<Id>,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: Some(123),
    };

    assert_eq!(ent.set_my_edge_id(None), Some(123));
    assert_eq!(ent.my_edge_id(), None);

    assert_eq!(ent.set_my_edge_id(Some(987)), None);
    assert_eq!(ent.my_edge_id(), Some(987));
}

#[test]
fn produces_setters_for_edge_ids_that_accepts_an_id_if_kind_is_one() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Id,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: 123,
    };

    assert_eq!(ent.set_my_edge_id(456), 123);
    assert_eq!(ent.my_edge_id(), 456);
}

#[test]
fn produces_setters_for_edge_ids_that_accepts_a_list_of_ids_if_kind_is_many() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Vec<Id>,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: vec![123, 456],
    };

    assert_eq!(ent.set_my_edge_ids(vec![987, 654, 321]), vec![123, 456]);
    assert_eq!(ent.my_edge_ids(), &[987, 654, 321]);
}

#[test]
fn produces_load_method_for_edge_of_kind_maybe_that_returns_an_option_of_ent() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Option<Id>,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: Some(1000),
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: None,
    };

    assert!(matches!(
        ent1.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent2.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));

    let database = InmemoryDatabase::default();
    ent1.connect(Box::from(database.clone()));
    ent2.connect(Box::from(database));

    ent1.clone().commit().expect("Failed to save ent1");
    ent2.clone().commit().expect("Failed to save ent2");

    assert_eq!(
        ent1.load_my_edge()
            .expect("Unexpected database failure loading edge for ent1")
            .expect("Missing ent for edge")
            .id,
        1000,
    );

    assert!(
        ent2.load_my_edge()
            .expect("Unexpected database failure loading edge for ent2")
            .is_none(),
        "Unexpectedly got ent for maybe edge of none",
    );
}

#[test]
fn produces_load_method_for_edge_of_kind_one_that_returns_a_single_ent() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Id,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: 1000,
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: 999,
    };

    assert!(matches!(
        ent1.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent2.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));

    let database = InmemoryDatabase::default();
    ent1.connect(Box::from(database.clone()));
    ent2.connect(Box::from(database));

    ent1.clone().commit().expect("Failed to save ent1");
    ent2.clone().commit().expect("Failed to save ent2");

    assert_eq!(
        ent1.load_my_edge()
            .expect("Unexpected database failure loading edge for ent1")
            .id,
        1000,
    );

    assert_eq!(
        ent2.load_my_edge()
            .expect("Unexpected database failure loading edge for ent2")
            .id,
        999,
    );
}

#[test]
fn produces_load_method_for_edge_of_kind_many_that_returns_zero_or_more_ents() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        my_edge: Vec<Id>,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: vec![1000, 1001],
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: vec![999, 1000],
    };

    assert!(matches!(
        ent1.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent2.load_my_edge(),
        Err(DatabaseError::Disconnected)
    ));

    let database = InmemoryDatabase::default();
    ent1.connect(Box::from(database.clone()));
    ent2.connect(Box::from(database));

    ent1.clone().commit().expect("Failed to save ent1");
    ent2.clone().commit().expect("Failed to save ent2");

    assert_eq!(
        ent1.load_my_edge()
            .expect("Unexpected database failure loading edge for ent1")
            .into_iter()
            .map(|ent| ent.id)
            .collect::<Vec<Id>>(),
        vec![1000],
    );

    assert_eq!(
        ent2.load_my_edge()
            .expect("Unexpected database failure loading edge for ent2")
            .into_iter()
            .map(|ent| ent.id)
            .collect::<Vec<Id>>(),
        vec![999, 1000],
    );
}

#[test]
fn supports_generic_ent_fields() {
    #[derive(Clone, Ent)]
    #[ent(typed_methods)]
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

        #[ent(field(mutable))]
        generic_field: T,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 123,
        last_updated: 456,
        generic_field: 0.5,
    };

    assert_eq!(ent.generic_field(), &0.5);
    assert_eq!(ent.set_generic_field(99.9), 0.5);
    assert_eq!(ent.generic_field, 99.9);
}
