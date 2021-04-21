use entity::EntQuery;

#[derive(EntQuery)]
enum TestEnt {
    A { a: u32 },
    B { b: u32 },
}

fn main() {}
