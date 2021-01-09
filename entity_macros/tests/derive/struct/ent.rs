use derivative::Derivative;
use entity::*;
use std::convert::TryFrom;

#[test]
fn supports_generic_fields() {
    #![allow(clippy::float_cmp)]

    #[derive(Clone, Ent)]
    struct TestEnt<T>
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
        generic_field: T,
    }

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 123,
        last_updated: 456,
        generic_field: 0.5,
    };
    assert_eq!(ent.generic_field, 0.5);
}

#[test]
fn id_should_return_copy_of_marked_id_field() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    assert_eq!(ent.id(), 999);
}

#[test]
fn set_id_should_update_the_marked_id_field() {
    #[derive(Clone, Ent)]
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

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    ent.set_id(123);
    assert_eq!(ent.id, 123);
}

#[test]
fn r#type_should_return_a_generated_type_using_module_path_and_ent_name() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    assert_eq!(ent.r#type(), concat!(module_path!(), "::", "TestEnt"));
}

#[test]
fn created_should_return_copy_of_marked_created_field() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 999,
        last_updated: 0,
    };

    assert_eq!(ent.created(), 999);
}

#[test]
fn last_updated_should_return_copy_of_marked_last_updated_field() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 999,
    };

    assert_eq!(ent.last_updated(), 999);
}

#[test]
fn field_definitions_should_return_list_of_definitions_for_ent_fields() {
    #[derive(Clone, Value)]
    struct CustomValue;

    #[derive(Clone, Ent)]
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
        a: u32,

        #[ent(field(indexed))]
        b: String,

        #[ent(field(mutable))]
        c: char,

        #[ent(field(indexed, mutable))]
        d: bool,

        #[ent(field)]
        e: CustomValue,
    }

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: 123,
        b: String::from("test"),
        c: 'z',
        d: true,
        e: CustomValue,
    };

    assert_eq!(
        ent.field_definitions(),
        vec![
            FieldDefinition::new_with_attributes(
                "a",
                NumberType::U32,
                vec![FieldAttribute::Immutable]
            ),
            FieldDefinition::new_with_attributes(
                "b",
                ValueType::Text,
                vec![FieldAttribute::Indexed, FieldAttribute::Immutable]
            ),
            FieldDefinition::new_with_attributes("c", PrimitiveValueType::Char, vec![]),
            FieldDefinition::new_with_attributes(
                "d",
                PrimitiveValueType::Bool,
                vec![FieldAttribute::Indexed]
            ),
            FieldDefinition::new_with_attributes(
                "e",
                ValueType::Custom,
                vec![FieldAttribute::Immutable]
            ),
        ]
    );
}

#[test]
fn field_should_return_abstract_value_if_exists() {
    #[derive(Clone, Debug, Value)]
    struct CustomValue;

    #[derive(Clone, Ent)]
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
        a: u32,

        #[ent(field(indexed))]
        b: String,

        #[ent(field(mutable))]
        c: char,

        #[ent(field(indexed, mutable))]
        d: bool,

        #[ent(field)]
        e: CustomValue,
    }

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: 123,
        b: String::from("test"),
        c: 'z',
        d: true,
        e: CustomValue,
    };

    assert_eq!(ent.field("a"), Some(Value::from(123u32)));
    assert_eq!(ent.field("b"), Some(Value::from("test")));
    assert_eq!(ent.field("c"), Some(Value::from('z')));
    assert_eq!(ent.field("d"), Some(Value::from(true)));
    assert_eq!(ent.field("e"), Some(Value::from(CustomValue)));
    assert_eq!(ent.field("f"), None);
}

#[test]
fn update_field_should_change_the_field_with_given_name_if_it_exists_to_value() {
    #[derive(Clone, Debug, PartialEq, Eq, Value)]
    struct CustomValue(usize);

    #[derive(Clone, Ent)]
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
        a: u32,

        #[ent(field(indexed))]
        b: String,

        #[ent(field(mutable))]
        c: char,

        #[ent(field(indexed, mutable))]
        d: bool,

        #[ent(field)]
        e: CustomValue,
    }

    let mut ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: 123,
        b: String::from("test"),
        c: 'z',
        d: true,
        e: CustomValue(123),
    };

    assert_eq!(
        ent.update_field("a", Value::from(234u32)).unwrap(),
        Value::from(123u32)
    );
    assert_eq!(ent.a, 234);

    assert_eq!(
        ent.update_field("b", Value::from("newtest")).unwrap(),
        Value::from("test")
    );
    assert_eq!(ent.b, "newtest");

    assert_eq!(
        ent.update_field("c", Value::from('$')).unwrap(),
        Value::from('z')
    );
    assert_eq!(ent.c, '$');

    assert_eq!(
        ent.update_field("d", Value::from(false)).unwrap(),
        Value::from(true)
    );
    assert_eq!(ent.d, false);

    assert_eq!(
        ent.update_field("e", Value::from(CustomValue(234)))
            .unwrap(),
        Value::from(CustomValue(123))
    );
    assert_eq!(ent.e, CustomValue(234));

    assert!(matches!(
        ent.update_field("id", Value::from(999usize)).unwrap_err(),
        EntMutationError::NoField { .. }
    ));

    assert!(matches!(
        ent.update_field("a", Value::from(999usize)).unwrap_err(),
        EntMutationError::WrongValueType { .. }
    ));
}

#[test]
fn edge_definitions_should_return_list_of_definitions_for_ent_edges() {
    #[derive(Clone, Ent)]
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
        a: Option<Id>,

        #[ent(edge(policy = "shallow", type = "TestEnt"))]
        b: Option<Id>,

        #[ent(edge(policy = "deep", type = "TestEnt"))]
        c: Option<Id>,

        #[ent(edge(policy = "nothing", type = "TestEnt"))]
        d: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        e: Id,

        #[ent(edge(policy = "shallow", type = "TestEnt"))]
        f: Id,

        #[ent(edge(policy = "deep", type = "TestEnt"))]
        g: Id,

        #[ent(edge(policy = "nothing", type = "TestEnt"))]
        h: Id,

        #[ent(edge(type = "TestEnt"))]
        i: Vec<Id>,

        #[ent(edge(policy = "shallow", type = "TestEnt"))]
        j: Vec<Id>,

        #[ent(edge(policy = "deep", type = "TestEnt"))]
        k: Vec<Id>,

        #[ent(edge(policy = "nothing", type = "TestEnt"))]
        l: Vec<Id>,
    }

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: None,
        b: None,
        c: None,
        d: None,
        e: 0,
        f: 0,
        g: 0,
        h: 0,
        i: vec![0],
        j: vec![0],
        k: vec![0],
        l: vec![0],
    };

    assert_eq!(
        ent.edge_definitions(),
        vec![
            EdgeDefinition::new_with_deletion_policy(
                "a",
                EdgeValueType::MaybeOne,
                EdgeDeletionPolicy::Nothing
            ),
            EdgeDefinition::new_with_deletion_policy(
                "b",
                EdgeValueType::MaybeOne,
                EdgeDeletionPolicy::ShallowDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "c",
                EdgeValueType::MaybeOne,
                EdgeDeletionPolicy::DeepDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "d",
                EdgeValueType::MaybeOne,
                EdgeDeletionPolicy::Nothing
            ),
            EdgeDefinition::new_with_deletion_policy(
                "e",
                EdgeValueType::One,
                EdgeDeletionPolicy::Nothing
            ),
            EdgeDefinition::new_with_deletion_policy(
                "f",
                EdgeValueType::One,
                EdgeDeletionPolicy::ShallowDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "g",
                EdgeValueType::One,
                EdgeDeletionPolicy::DeepDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "h",
                EdgeValueType::One,
                EdgeDeletionPolicy::Nothing
            ),
            EdgeDefinition::new_with_deletion_policy(
                "i",
                EdgeValueType::Many,
                EdgeDeletionPolicy::Nothing
            ),
            EdgeDefinition::new_with_deletion_policy(
                "j",
                EdgeValueType::Many,
                EdgeDeletionPolicy::ShallowDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "k",
                EdgeValueType::Many,
                EdgeDeletionPolicy::DeepDelete
            ),
            EdgeDefinition::new_with_deletion_policy(
                "l",
                EdgeValueType::Many,
                EdgeDeletionPolicy::Nothing
            ),
        ]
    );
}

#[test]
fn edge_should_return_abstract_value_if_exists() {
    #[derive(Clone, Ent)]
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
        a: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        b: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        c: Id,

        #[ent(edge(type = "TestEnt"))]
        d: Vec<Id>,
    }

    let ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: None,
        b: Some(3),
        c: 999,
        d: vec![1, 2, 3],
    };

    assert_eq!(ent.edge("a"), Some(EdgeValue::from(None)));
    assert_eq!(ent.edge("b"), Some(EdgeValue::from(Some(3))));
    assert_eq!(ent.edge("c"), Some(EdgeValue::from(999)));
    assert_eq!(ent.edge("d"), Some(EdgeValue::from(vec![1, 2, 3])));
    assert_eq!(ent.edge("e"), None);
}

#[test]
fn update_edge_should_change_the_edge_with_given_name_if_it_exists_to_value() {
    #[derive(Clone, Ent)]
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
        a: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        b: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        c: Id,

        #[ent(edge(type = "TestEnt"))]
        d: Vec<Id>,
    }

    let mut ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        a: None,
        b: Some(3),
        c: 999,
        d: vec![1, 2, 3],
    };

    assert_eq!(
        ent.update_edge("a", EdgeValue::from(Some(123))).unwrap(),
        EdgeValue::from(None)
    );
    assert_eq!(ent.a, Some(123));

    assert_eq!(
        ent.update_edge("b", EdgeValue::from(None)).unwrap(),
        EdgeValue::from(Some(3))
    );
    assert_eq!(ent.b, None);

    assert_eq!(
        ent.update_edge("c", EdgeValue::from(123)).unwrap(),
        EdgeValue::from(999)
    );
    assert_eq!(ent.c, 123);

    assert_eq!(
        ent.update_edge("d", EdgeValue::from(vec![4, 5])).unwrap(),
        EdgeValue::from(vec![1, 2, 3])
    );
    assert_eq!(ent.d, vec![4, 5]);

    assert!(matches!(
        ent.update_edge("e", EdgeValue::from(123)).unwrap_err(),
        EntMutationError::NoEdge { .. }
    ));

    assert!(matches!(
        ent.update_edge("a", EdgeValue::from(123)).unwrap_err(),
        EntMutationError::WrongEdgeValueType { .. }
    ));
}

#[test]
fn connect_should_replace_database_with_provided_one() {
    #[derive(Clone, Ent)]
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

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    let database = DatabaseRc::new(Box::new(InmemoryDatabase::default()));
    ent.connect(DatabaseRc::downgrade(&database));
    assert!(!WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
}

#[test]
fn disconnect_should_remove_any_associated_database() {
    #[derive(Clone, Ent)]
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

    let db = DatabaseRc::new(Box::from(InmemoryDatabase::default()));
    let mut ent = TestEnt {
        id: 999,
        database: DatabaseRc::downgrade(&db),
        created: 0,
        last_updated: 0,
    };

    ent.disconnect();
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
}

#[test]
fn is_connected_should_return_true_if_database_is_contained_within_ent() {
    #[derive(Clone, Ent)]
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

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    let db = DatabaseRc::new(Box::from(InmemoryDatabase::default()));
    assert_eq!(ent.is_connected(), false);
    ent.database = DatabaseRc::downgrade(&db);
    assert_eq!(ent.is_connected(), true);
}

#[test]
fn load_edge_should_return_new_copy_of_ents_pointed_to_by_ids() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug, PartialEq, Eq)]
    struct TestEnt {
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

        #[ent(edge(type = "TestEnt"))]
        parent: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        favorite: Id,

        #[ent(edge(type = "TestEnt"))]
        children: Vec<Id>,

        #[ent(field)]
        value: String,
    }

    let database = DatabaseRc::new(Box::from(InmemoryDatabase::default()));

    let mut ent1 = TestEnt {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        parent: None,
        favorite: 2,
        children: vec![2, 3],
        value: String::from("ent1"),
    };

    let mut ent2 = TestEnt {
        id: 2,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        parent: Some(1),
        favorite: 3,
        children: vec![],
        value: String::from("ent2"),
    };

    let mut ent3 = TestEnt {
        id: 3,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        parent: Some(1),
        favorite: 1,
        children: vec![],
        value: String::from("ent3"),
    };

    assert!(matches!(
        ent1.load_edge("parent"),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent1.load_edge("favorite"),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent1.load_edge("children"),
        Err(DatabaseError::Disconnected)
    ));
    assert!(matches!(
        ent1.load_edge("something"),
        Err(DatabaseError::Disconnected)
    ));

    ent1.connect(DatabaseRc::downgrade(&database));
    ent2.connect(DatabaseRc::downgrade(&database));
    ent3.connect(DatabaseRc::downgrade(&database));

    ent1.commit().expect("Failed to save Ent1");
    ent2.commit().expect("Failed to save Ent2");
    ent3.commit().expect("Failed to save Ent3");

    let children = ent1
        .load_edge("children")
        .expect("Failed to load ent1 children");
    assert_eq!(
        children
            .iter()
            .map(|ent| ent
                .as_any()
                .downcast_ref::<TestEnt>()
                .expect("Could not cast to TestEnt"))
            .collect::<Vec<&TestEnt>>(),
        vec![&ent2, &ent3],
    );

    let parent = ent2
        .load_edge("parent")
        .expect("Failed to load ent2 parent");
    assert_eq!(
        parent
            .iter()
            .map(|ent| ent
                .as_any()
                .downcast_ref::<TestEnt>()
                .expect("Could not cast to TestEnt"))
            .collect::<Vec<&TestEnt>>(),
        vec![&ent1],
    );

    let favorite = ent3
        .load_edge("favorite")
        .expect("Failed to load ent3 favorite");
    assert_eq!(
        favorite
            .iter()
            .map(|ent| ent
                .as_any()
                .downcast_ref::<TestEnt>()
                .expect("Could not cast to TestEnt"))
            .collect::<Vec<&TestEnt>>(),
        vec![&ent1],
    );

    assert!(matches!(
        ent1.load_edge("something"),
        Err(DatabaseError::MissingEdge { .. })
    ));
}

#[test]
fn refresh_should_update_ent_inplace_with_database_value() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug, PartialEq, Eq)]
    struct TestEnt {
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
        value: String,
    }

    let database = InmemoryDatabase::default();

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        value: String::from("test"),
    };

    assert!(matches!(ent.refresh(), Err(DatabaseError::Disconnected)));

    // Insert ent with same id that has some different values
    database
        .insert(Box::from(TestEnt {
            id: 999,
            database: WeakDatabaseRc::new(),
            created: 3,
            // NOTE: This will get replaced by the database
            last_updated: 0,
            value: String::from("different"),
        }))
        .expect("Failed to add ent to database");

    let database = DatabaseRc::new(Box::new(database));
    ent.connect(DatabaseRc::downgrade(&database));
    ent.refresh().expect("Failed to refresh ent");
    assert_eq!(ent.id, 999);
    assert!(
        !WeakDatabaseRc::ptr_eq(&ent.database, &WeakDatabaseRc::new()),
        "Non-assigned weak database ref found"
    );
    assert_eq!(ent.created, 3);
    assert!(ent.last_updated > 0);
    assert_eq!(ent.value, "different");
}

#[test]
fn commit_should_save_ent_to_database_and_update_id_if_it_was_changed() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug, PartialEq, Eq)]
    struct TestEnt {
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
    }

    let database = InmemoryDatabase::default();

    let mut ent = TestEnt {
        id: EPHEMERAL_ID,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    assert!(matches!(ent.commit(), Err(DatabaseError::Disconnected)));

    let database = DatabaseRc::new(Box::new(database));
    ent.connect(DatabaseRc::downgrade(&database));
    ent.commit().expect("Failed to commit ent");
    assert_ne!(ent.id, EPHEMERAL_ID, "Ephemeral id was not changed in ent");
    assert_eq!(
        database
            .get(ent.id)
            .expect("Unexpected database error")
            .is_some(),
        true,
    );
}

#[test]
fn remove_should_delete_ent_from_database() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug, PartialEq, Eq)]
    struct TestEnt {
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
    }

    let database = InmemoryDatabase::default();

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
    };

    assert!(matches!(ent.remove(), Err(DatabaseError::Disconnected)));

    let database = DatabaseRc::new(Box::new(database));
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
