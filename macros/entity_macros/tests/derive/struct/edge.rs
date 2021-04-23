use entity::*;
use entity_inmemory::InmemoryDatabase;
use std::collections::HashMap;

#[test]
fn produces_getters_for_edge_ids_that_returns_an_option_if_kind_is_maybe() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Option<Id>,
    }

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: Some(123),
    };

    assert_eq!(ent.my_edge_id(), Some(123));
}

#[test]
fn produces_getters_for_edge_ids_that_returns_the_id_if_kind_is_one() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Id,
    }

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: 123,
    };

    assert_eq!(ent.my_edge_id(), 123);
}

#[test]
fn produces_getters_for_edge_ids_that_returns_a_list_of_ids_if_kind_is_many() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Vec<Id>,
    }

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: vec![123, 456],
    };

    assert_eq!(ent.my_edge_ids(), &[123, 456]);
}

#[test]
fn produces_setters_for_edge_ids_that_accepts_an_optional_id_if_kind_is_maybe() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Option<Id>,
    }

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
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
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Id,
    }

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: 123,
    };

    assert_eq!(ent.set_my_edge_id(456), 123);
    assert_eq!(ent.my_edge_id(), 456);
}

#[test]
fn produces_setters_for_edge_ids_that_accepts_a_list_of_ids_if_kind_is_many() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Vec<Id>,
    }

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: vec![123, 456],
    };

    assert_eq!(ent.set_my_edge_ids(vec![987, 654, 321]), vec![123, 456]);
    assert_eq!(ent.my_edge_ids(), &[987, 654, 321]);
}

#[test]
fn produces_load_method_for_edge_of_kind_maybe_that_returns_an_option_of_ent() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Option<Id>,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: Some(1000),
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: WeakDatabaseRc::new(),
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

    let database = DatabaseRc::new(Box::new(InmemoryDatabase::default()));
    ent1.connect(DatabaseRc::downgrade(&database));
    ent2.connect(DatabaseRc::downgrade(&database));

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
fn supports_all_edge_kinds_for_edge_with_wrap_attr() {
    #[derive(Clone, Ent, EntTypedEdges, EntBuilder)]
    struct TestEnt1 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        other: Id,
    }

    #[derive(Clone, Ent, EntTypedEdges, EntBuilder)]
    struct TestEnt2 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        other: Option<Id>,
    }

    #[derive(Clone, Ent, EntTypedEdges, EntBuilder)]
    struct TestEnt3 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        others: Vec<Id>,
    }

    // Ent NOT included in enum
    #[derive(Clone, Ent, EntTypedEdges, EntBuilder)]
    struct TestEnt4 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    #[derive(Clone, Ent, EntWrapper)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
        Three(TestEnt3),
    }

    let db = DatabaseRc::new(Box::new(InmemoryDatabase::default()));
    let mut ent1 = TestEnt1::build()
        .database(DatabaseRc::downgrade(&db))
        .other(0)
        .finish_and_commit()
        .unwrap()
        .unwrap();
    let mut ent2 = TestEnt2::build()
        .database(DatabaseRc::downgrade(&db))
        .other(None)
        .finish_and_commit()
        .unwrap()
        .unwrap();
    let mut ent3 = TestEnt3::build()
        .database(DatabaseRc::downgrade(&db))
        .others(Vec::new())
        .finish_and_commit()
        .unwrap()
        .unwrap();
    let _ = TestEnt4::build()
        .database(DatabaseRc::downgrade(&db))
        .finish_and_commit()
        .unwrap()
        .unwrap();

    ent1.set_other_id(ent2.id());
    ent1.commit().unwrap();

    ent2.set_other_id(Some(ent3.id()));
    ent2.commit().unwrap();

    ent3.set_others_ids(vec![ent1.id(), ent2.id()]);
    ent3.commit().unwrap();

    let loaded_ent = ent1.load_other().unwrap();
    assert_eq!(loaded_ent.id(), ent2.id());
    assert!(matches!(loaded_ent, TestEntEnum::Two(_)));

    let loaded_ent = ent2.load_other().unwrap().unwrap();
    assert_eq!(loaded_ent.id(), ent3.id());
    assert!(matches!(loaded_ent, TestEntEnum::Three(_)));

    let loaded_ents = ent3
        .load_others()
        .unwrap()
        .into_iter()
        .map(|ent| {
            let id = ent.id();
            (id, ent)
        })
        .collect::<HashMap<Id, TestEntEnum>>();
    assert!(matches!(
        loaded_ents.get(&ent1.id()).unwrap(),
        TestEntEnum::One(_)
    ));
    assert!(matches!(
        loaded_ents.get(&ent2.id()).unwrap(),
        TestEntEnum::Two(_)
    ));
}

#[test]
fn produces_load_method_for_edge_of_kind_one_that_returns_a_single_ent() {
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Id,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: 1000,
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: WeakDatabaseRc::new(),
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

    let database = DatabaseRc::new(Box::new(InmemoryDatabase::default()));
    ent1.connect(DatabaseRc::downgrade(&database));
    ent2.connect(DatabaseRc::downgrade(&database));

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
    #[derive(Clone, Ent, EntTypedEdges)]
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
        my_edge: Vec<Id>,
    }

    let mut ent1 = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: vec![1000, 1001],
    };

    let mut ent2 = TestEnt {
        id: 1000,
        database: WeakDatabaseRc::new(),
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

    let database = DatabaseRc::new(Box::new(InmemoryDatabase::default()));
    ent1.connect(DatabaseRc::downgrade(&database));
    ent2.connect(DatabaseRc::downgrade(&database));

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
fn produces_load_method_for_edge_with_ent_wrapper_type_if_wrapper_attr_specified() {
    #[derive(Clone, Ent, EntTypedEdges)]
    struct TestEnt1 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        my_edge: Id,
    }

    #[derive(Clone, Ent, EntTypedEdges)]
    struct TestEnt2 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        my_edge: Id,
    }

    #[derive(Clone, Ent, EntTypedEdges)]
    struct TestEnt3 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEntEnum", wrap))]
        my_edge: Id,
    }

    #[derive(Clone, Ent, EntWrapper)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let database = DatabaseRc::new(Box::new(InmemoryDatabase::default()));

    let mut ent1 = TestEnt1 {
        id: 1,
        database: DatabaseRc::downgrade(&database),
        created: 0,
        last_updated: 0,
        my_edge: 2,
    };
    ent1.commit().unwrap();

    let mut ent2 = TestEnt2 {
        id: 2,
        database: DatabaseRc::downgrade(&database),
        created: 0,
        last_updated: 0,
        my_edge: 3,
    };
    ent2.commit().unwrap();

    let mut ent3 = TestEnt3 {
        id: 3,
        database: DatabaseRc::downgrade(&database),
        created: 0,
        last_updated: 0,
        my_edge: 1,
    };
    ent3.commit().unwrap();

    assert!(matches!(ent1.load_my_edge(), Ok(TestEntEnum::Two(_))));
    assert!(matches!(
        ent2.load_my_edge(),
        Err(DatabaseError::BrokenEdge { .. })
    ));
    assert!(matches!(ent3.load_my_edge(), Ok(TestEntEnum::One(_))));
}
