use entity::{simple_ent, Ent};

#[simple_ent]
#[derive(Clone, Ent)]
pub struct SimpleEnt {
    #[ent(field)]
    created: usize,
}

fn main() {}
