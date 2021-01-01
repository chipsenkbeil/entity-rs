#[entity::simple_ent]
struct TestEnt1 {
    field1: bool,
}

#[entity::simple_ent]
struct TestEnt2 {
    field1: usize,
    field2: f32,
}

#[test]
fn adds_derive_clone_ent_when_missing() {
    #[entity::simple_ent]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let ent = TestEntEnum::One(TestEnt1 {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    });

    use entity::Ent;
    assert_eq!(ent.id(), 123);
    assert!(!ent.is_connected());
    assert_eq!(ent.created(), 456);
    assert_eq!(ent.last_updated(), 789);
}

#[test]
fn fills_in_derive_clone_when_missing() {
    #[entity::simple_ent]
    #[derive(entity::Ent)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let ent = TestEntEnum::One(TestEnt1 {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    });

    use entity::Ent;
    assert_eq!(ent.id(), 123);
    assert!(!ent.is_connected());
    assert_eq!(ent.created(), 456);
    assert_eq!(ent.last_updated(), 789);
}

#[test]
fn fills_in_derive_ent_when_missing() {
    #[entity::simple_ent]
    #[derive(std::clone::Clone)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let ent = TestEntEnum::One(TestEnt1 {
        id: 123,
        database: entity::WeakDatabaseRc::new(),
        created: 456,
        last_updated: 789,
        field1: true,
    });

    use entity::Ent;
    assert_eq!(ent.id(), 123);
    assert!(!ent.is_connected());
    assert_eq!(ent.created(), 456);
    assert_eq!(ent.last_updated(), 789);
}
