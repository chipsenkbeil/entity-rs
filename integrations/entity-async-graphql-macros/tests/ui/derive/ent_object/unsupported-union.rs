use entity_async_graphql_macros::EntObject;

#[derive(EntObject)]
union TestEnt {
    a: u32,
    b: char,
}

fn main() {}
