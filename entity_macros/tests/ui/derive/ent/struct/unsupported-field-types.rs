use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Ent)]
struct TestEnt<'a> {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field)]
    a: [u32; 3],

    #[ent(field)]
    b: fn(u32) -> u32,

    #[ent(field)]
    c: *const u32,

    #[ent(field)]
    d: *mut u32,

    #[ent(field)]
    e: &'a u32,

    #[ent(field)]
    f: &'a mut u32,

    #[ent(field)]
    g: &'a [u32],

    #[ent(field)]
    h: (u32, u32),

    #[ent(field)]
    i: Vec<&'a u32>,

    #[ent(field)]
    j: std::collections::HashMap<String, &'a u32>,

    #[ent(field)]
    k: Option<&'a u32>,
}

fn main() {}
