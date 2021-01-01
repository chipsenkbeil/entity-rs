use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(database)]
    database2: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
