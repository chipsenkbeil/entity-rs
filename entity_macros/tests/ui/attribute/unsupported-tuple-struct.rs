use entity::{include_ent_core, Ent};

#[include_ent_core]
#[derive(Clone, Ent)]
pub struct SimpleEnt(#[ent(field)] String, #[ent(field)] u32);

fn main() {}
