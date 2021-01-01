use entity::{Ent, Id, WeakDatabaseRc};

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

    #[ent(edge(type = "TestEnt", something))]
    my_edge: Id,
}

fn main() {}
