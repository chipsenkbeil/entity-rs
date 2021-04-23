use entity::*;

#[test]
fn implements_debug_excluding_database() {
    #[derive(Clone, EntDebug)]
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
        field1: u32,

        #[ent(edge(type = "TestEnt"))]
        other: Id,
    }

    let ent = TestEnt {
        id: 1,
        database: WeakDatabaseRc::new(),
        created: 2,
        last_updated: 3,
        field1: 4,
        other: 5,
    };

    assert_eq!(
        format!("{:?}", ent),
        "TestEnt { id: 1, created: 2, last_updated: 3, field1: 4, other: 5 }"
    );
}
