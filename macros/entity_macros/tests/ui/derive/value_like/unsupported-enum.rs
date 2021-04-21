use entity::ValueLike;

#[derive(ValueLike)]
enum TestValue {
    A(u32),
    B(String),
}

fn main() {}
