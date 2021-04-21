use entity_async_graphql_derive::EntObject;

#[derive(EntObject)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
