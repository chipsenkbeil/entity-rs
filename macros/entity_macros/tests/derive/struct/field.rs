use entity::*;

#[test]
fn produces_getters_for_fields_that_returns_references() {
    #[derive(Clone, Ent, EntTypedFields)]
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
        my_field1: u32,

        #[ent(field)]
        my_field2: String,
    }

    let ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
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
    #[derive(Clone, Ent, EntTypedFields)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

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
        database: WeakDatabaseRc::new(),
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
fn supports_generic_ent_fields() {
    #![allow(clippy::float_cmp)]

    #[derive(Clone, Ent, EntTypedFields)]
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

        #[ent(field(mutable))]
        generic_field: T,
    }

    let mut ent = TestEnt {
        id: 999,
        database: WeakDatabaseRc::new(),
        created: 123,
        last_updated: 456,
        generic_field: 0.5,
    };

    assert_eq!(ent.generic_field(), &0.5);
    assert_eq!(ent.set_generic_field(99.9), 0.5);
    assert_eq!(ent.generic_field, 99.9);
}
