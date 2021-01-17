use entity::Ent;

#[derive(Ent)]
struct TestEnt {
    #[ent(id, database, created, last_updated)]
    some_field: u32,
}

fn main() {}
