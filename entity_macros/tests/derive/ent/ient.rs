use entity::{
    Database, EdgeDefinition, EdgeDeletionPolicy, EdgeValue, EdgeValueType, Ent, EntMutationError,
    FieldAttribute, FieldDefinition, IEnt, Id, InmemoryDatabase, NumberType, PrimitiveValueType,
    Value, ValueType,
};

#[test]
fn id_should_return_copy_of_marked_id_field() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: 999,
        database: None,
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
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
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
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
    };

    assert_eq!(TEST_ENT_TYPE, concat!(module_path!(), "::", "TestEnt"));
    assert_eq!(ent.r#type(), concat!(module_path!(), "::", "TestEnt"));
}

#[test]
fn created_should_return_copy_of_marked_created_field() {
    #[derive(Clone, Ent)]
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

    let ent = TestEnt {
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = TestEnt {
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

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
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

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
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

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
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(edge(type = "TestEnt"))]
        a: Option<Id>,

        #[ent(edge(shallow, type = "TestEnt"))]
        b: Option<Id>,

        #[ent(edge(deep, type = "TestEnt"))]
        c: Option<Id>,

        #[ent(edge(nothing, type = "TestEnt"))]
        d: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        e: Id,

        #[ent(edge(shallow, type = "TestEnt"))]
        f: Id,

        #[ent(edge(deep, type = "TestEnt"))]
        g: Id,

        #[ent(edge(nothing, type = "TestEnt"))]
        h: Id,

        #[ent(edge(type = "TestEnt"))]
        i: Vec<Id>,

        #[ent(edge(shallow, type = "TestEnt"))]
        j: Vec<Id>,

        #[ent(edge(deep, type = "TestEnt"))]
        k: Vec<Id>,

        #[ent(edge(nothing, type = "TestEnt"))]
        l: Vec<Id>,
    }

    let ent = TestEnt {
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

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
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

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
        id: 0,
        database: None,
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
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
    };

    ent.connect(Box::from(InmemoryDatabase::default()));
    assert!(ent.database.is_some());
}

#[test]
fn disconnect_should_remove_any_associated_database() {
    #[derive(Clone, Ent)]
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

    let mut ent = TestEnt {
        id: 999,
        database: Some(Box::from(InmemoryDatabase::default())),
        created: 0,
        last_updated: 0,
    };

    ent.disconnect();
    assert!(ent.database.is_none());
}

#[test]
fn is_connected_should_return_true_if_database_is_contained_within_ent() {
    #[derive(Clone, Ent)]
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

    let mut ent = TestEnt {
        id: 999,
        database: None,
        created: 0,
        last_updated: 0,
    };

    assert_eq!(ent.is_connected(), false);
    ent.database = Some(Box::from(InmemoryDatabase::default()));
    assert_eq!(ent.is_connected(), true);
}

#[test]
fn load_edge_should_return_new_copy_of_ents_pointed_to_by_ids() {
    todo!();
}

#[test]
fn refresh_should_update_ent_inplace_with_database_value() {
    todo!();
}

#[test]
fn commit_should_save_ent_to_database() {
    todo!();
}

#[test]
fn remove_should_delete_ent_from_database() {
    todo!();
}
