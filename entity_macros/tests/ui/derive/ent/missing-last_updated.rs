use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,
}

fn main() {}
