use entity::EntWrapper;

#[derive(EntWrapper)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
