use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
#[ent(no_builder)]
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

fn main() {
    let _ = TestEntBuilder::default();
}
