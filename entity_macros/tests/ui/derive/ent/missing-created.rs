use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
