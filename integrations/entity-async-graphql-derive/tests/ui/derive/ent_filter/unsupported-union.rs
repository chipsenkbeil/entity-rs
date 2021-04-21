use entity_async_graphql_derive::EntFilter;

#[derive(EntFilter)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
