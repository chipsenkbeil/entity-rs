use entity::{IntoValue, ValueLike};

#[derive(ValueLike, IntoValue)]
union TestValue {
    a: u32,
    b: char,
}

fn main() {}
