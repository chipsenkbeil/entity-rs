use entity::EntBuilder;

#[derive(EntBuilder)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
