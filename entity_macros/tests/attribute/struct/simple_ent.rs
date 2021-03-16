use entity::{simple_ent, Ent, Id, WeakDatabaseRc};

#[test]
fn adds_derive_clone_ent_when_missing() {
    #[simple_ent]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_derive_clone_when_missing() {
    #[simple_ent]
    #[derive(entity::Ent)]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_derive_ent_when_missing() {
    #[simple_ent]
    #[derive(std::clone::Clone)]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_ent_created_field_when_missing() {
    #[simple_ent]
    #[derive(Clone, Ent)]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_ent_database_field_when_missing() {
    #[simple_ent]
    #[derive(Clone, Ent)]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_ent_id_field_when_missing() {
    #[simple_ent]
    #[derive(Clone, Ent)]
    struct SimpleEnt {
        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_ent_last_updated_field_when_missing() {
    #[simple_ent]
    #[derive(Clone, Ent)]
    struct SimpleEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,
    }

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn fills_in_everything_missing() {
    #[simple_ent]
    struct SimpleEnt {}

    let ent = SimpleEnt {
        id: 123,
        database: WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
    };

    assert_eq!(ent.id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.created, 456);
    assert_eq!(ent.last_updated, 789);
}

#[test]
fn supports_renaming_ent_fields() {
    #[simple_ent(
        id = "my_id",
        database = "my_database",
        created = "my_created",
        last_updated = "my_last_updated"
    )]
    struct SimpleEnt {}

    let ent = SimpleEnt {
        my_id: 123,
        my_database: WeakDatabaseRc::new(),
        my_created: 456,
        my_last_updated: 789,
    };

    assert_eq!(ent.my_id, 123);
    assert!(WeakDatabaseRc::ptr_eq(
        &ent.my_database,
        &WeakDatabaseRc::new()
    ));
    assert_eq!(ent.my_created, 456);
    assert_eq!(ent.my_last_updated, 789);
}

#[test]
fn supports_using_struct_field_type_for_ent_type() {
    #[simple_ent]
    struct SimpleEnt {
        #[ent(edge)]
        maybe_edge: Option<SimpleEnt>,

        #[ent(edge(policy = "shallow"))]
        edge: SimpleEnt,

        #[ent(edge)]
        many_edges: Vec<SimpleEnt>,

        #[ent(edge(type = "SimpleEnt"))]
        explicit_edge: Id,
    }

    let ent = SimpleEnt {
        id: 0,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        maybe_edge: Some(123),
        edge: 456,
        many_edges: vec![1, 2, 3, 4],
        explicit_edge: 789,
    };

    assert_eq!(ent.maybe_edge, Some(123));
    assert_eq!(ent.edge, 456);
    assert_eq!(ent.many_edges, vec![1, 2, 3, 4]);
    assert_eq!(ent.explicit_edge, 789);
}

#[test]
fn supports_common_field_types() {
    use std::collections::*;

    // Validate via compilation, no need to test anything else
    #[simple_ent]
    struct TestEnt {
        a1: bool,
        a2: char,
        a3: u8,
        a4: u16,
        a5: u32,
        a6: u64,
        a7: u128,
        a8: usize,
        a9: i8,
        a10: i16,
        a11: i32,
        a12: i64,
        a13: i128,
        a14: isize,
        a15: f32,
        a16: f64,
        b1: Vec<String>,
        b2: VecDeque<String>,
        b3: LinkedList<String>,
        b4: BinaryHeap<String>,
        b5: HashSet<String>,
        b6: BTreeSet<String>,
        b7: HashMap<String, String>,
        b8: BTreeMap<String, String>,
    }
}
