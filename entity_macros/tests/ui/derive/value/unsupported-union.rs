use entity::Value;

#[derive(Value)]
union TestValue {
    a: u32,
    b: char,
}

fn main() {}
