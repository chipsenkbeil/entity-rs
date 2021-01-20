use entity::EntQuery;

#[derive(EntQuery)]
enum TestEnt {
    A(u32, u32),
    B(u32, u32),
}

fn main() {}
