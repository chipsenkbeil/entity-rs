use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent("something")]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,
}

fn main() {}
