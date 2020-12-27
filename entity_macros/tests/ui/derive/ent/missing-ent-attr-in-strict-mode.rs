use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
#[ent(strict)]
struct TestEnt {
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
