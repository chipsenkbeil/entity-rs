use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
struct TestEnt<'a> {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    my_field: &'a u32,
}

fn main() {}
