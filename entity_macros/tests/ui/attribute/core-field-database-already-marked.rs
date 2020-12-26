use entity::{include_ent_core, Database, Ent};

#[include_ent_core]
#[derive(Clone, Ent)]
pub struct SimpleEnt {
    #[ent(database)]
    something: Option<Box<dyn Database>>,
}

fn main() {}
