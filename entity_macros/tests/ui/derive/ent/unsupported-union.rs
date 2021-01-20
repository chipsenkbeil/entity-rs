use entity::Ent;

#[derive(Ent)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
