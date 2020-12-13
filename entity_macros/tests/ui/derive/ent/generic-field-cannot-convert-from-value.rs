use entity::{Database, Ent, Id, Value};

#[derive(Clone, Ent)]
struct TestEnt<T: Clone + Into<Value> + 'static> {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    my_field: T,
}

fn main() {}
