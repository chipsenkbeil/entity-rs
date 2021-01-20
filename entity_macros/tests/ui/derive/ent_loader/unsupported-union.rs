use entity::EntLoader;

#[derive(EntLoader)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
