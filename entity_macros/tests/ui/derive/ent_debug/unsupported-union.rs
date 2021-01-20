use entity::EntDebug;

#[derive(EntDebug)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
