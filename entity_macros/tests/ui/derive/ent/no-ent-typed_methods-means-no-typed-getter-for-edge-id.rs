use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
#[ent(no_typed_methods)]
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

fn main() {
    let ent = TestEnt {
        id: 0,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_edge: 0,
    };

    let _ = ent.my_edge_id();
}
