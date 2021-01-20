use entity::AsyncGraphqlEntFilter;

#[derive(AsyncGraphqlEntFilter)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
