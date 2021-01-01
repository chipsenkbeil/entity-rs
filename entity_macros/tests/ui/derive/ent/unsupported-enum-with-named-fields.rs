use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
enum TestEnt {
    Variant {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    },
}

fn main() {}
