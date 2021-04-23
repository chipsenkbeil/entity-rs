use entity::EntTypedFields;

#[derive(EntTypedFields)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
