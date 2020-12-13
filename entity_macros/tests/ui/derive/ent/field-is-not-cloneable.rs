use entity::{Database, Ent, Id, Value};

#[derive(Value)]
struct CustomField;

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    custom: CustomField,
}

fn main() {}
