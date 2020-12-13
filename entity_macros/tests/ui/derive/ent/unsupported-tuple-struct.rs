use entity::{Database, Ent, Id};

#[derive(Clone, Ent)]
struct TestEnt(
    #[ent(id)] Id,
    #[ent(database)] Option<Box<dyn Database>>,
    #[ent(created)] u64,
    #[ent(last_updated)] u64,
);

fn main() {}
