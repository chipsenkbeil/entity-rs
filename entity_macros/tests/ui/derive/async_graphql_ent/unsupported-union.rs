use entity::AsyncGraphqlEnt;

#[derive(AsyncGraphqlEnt)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
