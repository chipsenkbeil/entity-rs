use entity::EntType;

#[derive(EntType)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
