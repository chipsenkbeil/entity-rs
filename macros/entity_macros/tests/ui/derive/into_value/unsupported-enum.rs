use entity::{IntoValue, ValueLike};

#[derive(ValueLike, IntoValue)]
enum TestValue {
    A(u32),
    B(String),
}

fn main() {}
