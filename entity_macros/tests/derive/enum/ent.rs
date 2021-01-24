use derivative::Derivative;
use entity::*;

#[derive(Clone, Derivative, Ent)]
#[derivative(Debug, PartialEq)]
struct TestEnt1 {
    #[ent(id)]
    id: Id,

    #[derivative(Debug = "ignore")]
    #[derivative(PartialEq = "ignore")]
    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[derivative(PartialEq = "ignore")]
    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    field1: usize,

    #[ent(edge(type = "TestEnt2"))]
    other: Id,
}

#[derive(Clone, Derivative, Ent)]
#[derivative(Debug, PartialEq)]
struct TestEnt2 {
    #[ent(id)]
    id: Id,

    #[derivative(Debug = "ignore")]
    #[derivative(PartialEq = "ignore")]
    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[derivative(PartialEq = "ignore")]
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

#[derive(Clone, Debug, PartialEq, Ent)]
enum TestEnt {
    One(TestEnt1),
    Two(TestEnt2),
}

#[test]
fn supports_generic_fields() {
    #![allow(clippy::float_cmp)]

    #[derive(Clone, Ent)]
    struct GenericTestEnt<T>
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
        generic_field: T,
    }

    #[derive(Clone, Ent)]
    enum GenericTestEntEnum<T>
    where
        T: ValueLike + Clone + Send + Sync + 'static,
    {
        Choice(GenericTestEnt<T>),
    }

    let ent = GenericTestEntEnum::Choice(GenericTestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 123,
        last_updated: 456,
        generic_field: 0.5,
    });

    match ent {
        GenericTestEntEnum::Choice(x) => assert_eq!(x.generic_field, 0.5),
    }
}

#[test]
fn id_should_return_copy_of_marked_id_field() {
    let ent = TestEnt::One(TestEnt1 {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 999,
        other: 2,
    });

    assert_eq!(ent.id(), 1);
}

#[test]
fn set_id_should_update_the_marked_id_field() {
    let mut ent = TestEnt::One(TestEnt1 {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 999,
        other: 2,
    });

    ent.set_id(123);
    assert_eq!(ent.id(), 123);
}

#[test]
fn r#type_should_return_a_generated_type_using_module_path_and_enum_name() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 999,
        other: 2,
    });

    assert_eq!(ent.r#type(), concat!(module_path!(), "::", "TestEnt"));
}

#[test]
fn created_should_return_copy_of_marked_created_field() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 999,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert_eq!(ent.created(), 999);
}

#[test]
fn last_updated_should_return_copy_of_marked_last_updated_field() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 999,
        field1: 1,
        other: 2,
    });

    assert_eq!(ent.last_updated(), 999);
}

#[test]
fn field_definitions_should_return_list_of_definitions_for_ent_fields() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 999,
        field1: 1,
        other: 2,
    });

    assert_eq!(
        ent.field_definitions(),
        vec![FieldDefinition::new_with_attributes(
            "field1",
            NumberType::Usize,
            vec![FieldAttribute::Immutable]
        ),]
    );
}

#[test]
fn field_should_return_abstract_value_if_exists() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 999,
        field1: 1,
        other: 2,
    });

    assert_eq!(ent.field("field1"), Some(Value::from(1)));
    assert_eq!(ent.field("field2"), None);
}

#[test]
fn update_field_should_change_the_field_with_given_name_if_it_exists_to_value() {
    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 999,
        field1: 1,
        other: 2,
    });

    assert_eq!(
        ent.update_field("field1", Value::from(123usize)).unwrap(),
        Value::from(1)
    );
    match &ent {
        TestEnt::One(x) => assert_eq!(x.field1, 123),
        x => panic!("Wrong ent found: {:?}", x),
    }

    assert!(matches!(
        ent.update_field("id", Value::from(999usize)).unwrap_err(),
        EntMutationError::NoField { .. }
    ));

    assert!(matches!(
        ent.update_field("field1", Value::from("test")).unwrap_err(),
        EntMutationError::WrongValueType { .. }
    ));
}

#[test]
fn edge_definitions_should_return_list_of_definitions_for_ent_edges() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert_eq!(
        ent.edge_definitions(),
        vec![EdgeDefinition::new_with_deletion_policy(
            "other",
            EdgeValueType::One,
            EdgeDeletionPolicy::Nothing
        ),]
    );
}

#[test]
fn edge_should_return_abstract_value_if_exists() {
    let ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert_eq!(ent.edge("other"), Some(EdgeValue::from(2)));
    assert_eq!(ent.edge("missing"), None);
}

#[test]
fn update_edge_should_change_the_edge_with_given_name_if_it_exists_to_value() {
    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert_eq!(
        ent.update_edge("other", EdgeValue::from(123)).unwrap(),
        EdgeValue::from(2)
    );
    match &ent {
        TestEnt::One(x) => assert_eq!(x.other, 123),
        x => panic!("Wrong ent found: {:?}", x),
    }

    assert!(matches!(
        ent.update_edge("missing", EdgeValue::from(123)).unwrap_err(),
        EntMutationError::NoEdge { .. }
    ));

    assert!(matches!(
        ent.update_edge("other", EdgeValue::from(vec![1, 2, 3])).unwrap_err(),
        EntMutationError::WrongEdgeValueType { .. }
    ));
}

#[test]
fn connect_should_replace_database_with_provided_one() {
    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    let db = DatabaseRc::new(Box::from(InmemoryDatabase::default()));
    ent.connect(DatabaseRc::downgrade(&db));
    match ent {
        TestEnt::One(x) => assert!(WeakDatabaseRc::ptr_eq(
            &x.database,
            &DatabaseRc::downgrade(&db)
        )),
        x => panic!("Wrong ent found: {:?}", x),
    }
}

#[test]
fn disconnect_should_remove_any_associated_database() {
    let db = DatabaseRc::new(Box::from(InmemoryDatabase::default()));
    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: DatabaseRc::downgrade(&db),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    ent.disconnect();
    match ent {
        TestEnt::One(x) => assert!(WeakDatabaseRc::ptr_eq(&x.database, &WeakDatabaseRc::new())),
        x => panic!("Wrong ent found: {:?}", x),
    }
}

#[test]
fn is_connected_should_return_true_if_database_is_contained_within_ent() {
    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    let db = DatabaseRc::new(Box::from(InmemoryDatabase::default()));
    assert_eq!(ent.is_connected(), false);
    match &mut ent {
        TestEnt::One(x) => x.database = DatabaseRc::downgrade(&db),
        x => panic!("Wrong ent found: {:?}", x),
    }
    assert_eq!(ent.is_connected(), true);
}

#[test]
fn load_edge_should_return_new_copy_of_ents_pointed_to_by_ids() {
    let database = DatabaseRc::new(Box::from(InmemoryDatabase::default()));

    let mut ent1 = TestEnt::One(TestEnt1 {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    let mut ent2 = TestEnt::Two(TestEnt2 {
        id: 2,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        field2: String::from("test"),
        maybe_other: Some(1),
        dups: vec![],
    });

    assert!(matches!(
        ent1.load_edge("other"),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent1.load_edge("missing"),
        Err(DatabaseError::Disconnected)
    ));

    ent1.connect(DatabaseRc::downgrade(&database));
    ent2.connect(DatabaseRc::downgrade(&database));

    ent1.commit().expect("Failed to save Ent1");
    ent2.commit().expect("Failed to save Ent2");

    let other = ent1.load_edge("other").expect("Failed to load ent1 other");
    assert_eq!(
        other
            .iter()
            .map(|ent| TestEnt::Two(
                ent.to_ent::<TestEnt2>()
                    .expect("Could not cast to TestEnt2")
            ))
            .collect::<Vec<TestEnt>>(),
        vec![ent2],
    );

    assert!(matches!(
        ent1.load_edge("missing"),
        Err(DatabaseError::MissingEdge { .. })
    ));
}

#[test]
fn refresh_should_update_ent_within_variant_inplace_with_database_value() {
    let database = DatabaseRc::new(Box::from(InmemoryDatabase::default()));

    let mut ent = TestEnt::One(TestEnt1 {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert!(matches!(ent.refresh(), Err(DatabaseError::Disconnected)));

    // Insert ent with same id that has some different values
    database
        .insert(Box::from(TestEnt1 {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 3,
            // NOTE: This will get replaced by the database
            last_updated: 0,
            field1: 999,
            other: 123,
        }))
        .expect("Failed to add ent to database");

    ent.connect(DatabaseRc::downgrade(&database));
    ent.refresh().expect("Failed to refresh ent");
    match ent {
        TestEnt::One(x) => {
            assert_eq!(x.id, 1);
            assert!(WeakDatabaseRc::ptr_eq(
                &x.database,
                &DatabaseRc::downgrade(&database)
            ));
            assert_eq!(x.created, 3);
            assert!(x.last_updated > 0);
            assert_eq!(x.field1, 999);
            assert_eq!(x.other, 123);
        }
        x => panic!("Wrong ent found: {:?}", x),
    }
}

#[test]
fn commit_should_save_ent_to_database_and_update_id_if_it_was_changed() {
    let database = DatabaseRc::new(Box::from(InmemoryDatabase::default()));

    let mut ent = TestEnt::One(TestEnt1 {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert!(matches!(ent.commit(), Err(DatabaseError::Disconnected)));

    ent.connect(DatabaseRc::downgrade(&database));
    ent.commit().expect("Failed to commit ent");
    assert_ne!(
        ent.id(),
        EPHEMERAL_ID,
        "Ephemeral id was not changed in ent"
    );
    assert_eq!(
        database
            .get(ent.id())
            .expect("Unexpected database error")
            .is_some(),
        true,
    );
}

#[test]
fn remove_should_delete_ent_from_database() {
    let database = DatabaseRc::new(Box::from(InmemoryDatabase::default()));

    let mut ent = TestEnt::One(TestEnt1 {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        field1: 1,
        other: 2,
    });

    assert!(matches!(ent.remove(), Err(DatabaseError::Disconnected)));

    ent.connect(DatabaseRc::downgrade(&database));
    assert_eq!(ent.remove().expect("Failed to remove ent"), false);
    assert_eq!(
        database.get(999).expect("Failed to get ent").is_none(),
        true,
        "Ent unexpectedly in database",
    );

    ent.commit().expect("Failed to insert ent into database");
    assert_eq!(
        database.get(999).expect("Failed to get ent").is_some(),
        true,
        "Ent unexpectedly not in database",
    );
    assert_eq!(ent.remove().expect("Failed to remove ent"), true);
    assert_eq!(
        database.get(999).expect("Failed to get ent").is_none(),
        true,
        "Ent unexpectedly in database",
    );
}
