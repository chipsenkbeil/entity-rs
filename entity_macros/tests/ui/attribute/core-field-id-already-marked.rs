use entity::{include_ent_core, Ent, Id};

#[include_ent_core]
#[derive(Clone, Ent)]
pub struct SimpleEnt {
    #[ent(id)]
    something: Id,
}

fn main() {}
