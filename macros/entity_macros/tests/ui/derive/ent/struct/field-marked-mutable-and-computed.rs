use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field(mutable, computed = "123"))]
    computed_field: Option<u32>,
}

fn main() {}
