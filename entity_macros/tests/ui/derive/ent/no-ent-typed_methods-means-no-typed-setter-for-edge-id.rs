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

    #[ent(edge(type = "TestEnt"))]
    my_edge: Id,
}

fn main() {
    let ent = TestEnt {
        id: 0,
        database: None,
        created: 0,
        last_updated: 0,
        my_edge: 0,
    };

    let _ = ent.set_my_edge_id(3);
}
