use entity::{include_ent_core, Ent};

#[include_ent_core]
#[derive(Clone, Ent)]
pub struct SimpleEnt {
    #[ent(last_updated)]
    something: u64,
}

fn main() {}
