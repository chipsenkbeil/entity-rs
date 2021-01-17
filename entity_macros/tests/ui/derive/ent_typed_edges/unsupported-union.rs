use entity::EntTypedEdges;

#[derive(EntTypedEdges)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
