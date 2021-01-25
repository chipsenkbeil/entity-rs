use entity::ValueLike;

#[derive(ValueLike)]
union TestValue {
    a: u32,
    b: char,
}

fn main() {}
