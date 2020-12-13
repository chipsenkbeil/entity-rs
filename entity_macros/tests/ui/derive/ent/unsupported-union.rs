use entity::Ent;

#[derive(Copy, Clone, Ent)]
union TestEnt {
    a: u32,
    b: i32,
}

fn main() {}
