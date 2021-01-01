use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
struct TestEnt(
    #[ent(id)] Id,
    #[ent(database)] WeakDatabaseRc,
    #[ent(created)] u64,
    #[ent(last_updated)] u64,
);

fn main() {}
