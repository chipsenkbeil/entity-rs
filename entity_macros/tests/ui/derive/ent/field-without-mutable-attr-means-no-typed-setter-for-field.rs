use entity::{Database, Ent, Id};

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
    my_field: u32,
}

fn main() {
    let mut ent = TestEnt {
        id: 0,
        database: None,
        created: 0,
        last_updated: 0,
        my_field: 0,
    };

    let _ = ent.my_field();
    ent.set_my_field(3).unwrap();
}
