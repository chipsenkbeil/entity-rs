use entity::{TryFromValue, ValueLike};

#[derive(ValueLike, TryFromValue)]
union TestValue {
    a: u32,
    b: char,
}

fn main() {}
