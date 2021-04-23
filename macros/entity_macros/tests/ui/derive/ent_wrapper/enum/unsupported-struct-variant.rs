use entity::EntWrapper;

#[derive(EntWrapper)]
enum TestEnt {
    A { a: u32 },
    B { b: u32 },
}

fn main() {}
