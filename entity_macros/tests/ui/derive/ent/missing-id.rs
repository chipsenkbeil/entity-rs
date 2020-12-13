use entity::{Database, Ent};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
