use entity_async_graphql_macros::EntFilter;

#[derive(EntFilter)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
