use entity::Value;

#[derive(Value)]
enum TestValue {
    A(u32),
    B(String),
}

fn main() {}
