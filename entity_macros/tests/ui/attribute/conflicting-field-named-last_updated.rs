use entity::{include_ent_core, Ent};

#[include_ent_core]
#[derive(Clone, Ent)]
pub struct SimpleEnt {
    #[ent(field)]
    last_updated: usize,
}

fn main() {}
