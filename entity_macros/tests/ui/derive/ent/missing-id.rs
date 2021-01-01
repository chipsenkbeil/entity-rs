use entity::{Ent, WeakDatabaseRc};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
