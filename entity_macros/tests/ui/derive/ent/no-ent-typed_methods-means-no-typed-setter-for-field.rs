use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
#[ent(no_typed_methods)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field(mutable))]
    my_field: u32,
}

fn main() {
    let mut ent = TestEnt {
        id: 0,
        database: WeakDatabaseRc::new(),
        created: 0,
        last_updated: 0,
        my_field: 0,
    };

    ent.set_my_field(3).unwrap();
}
