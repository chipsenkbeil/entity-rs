use entity::EntQuery;

#[derive(EntQuery)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
