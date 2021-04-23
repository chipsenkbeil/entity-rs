use entity::{TryFromValue, ValueLike};

#[derive(ValueLike, TryFromValue)]
enum TestValue {
    A(u32),
    B(String),
}

fn main() {}
