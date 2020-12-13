#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}

// #[test]
// fn test_test_test() {
//     use entity::{Database, Ent, Id, Value};
//     use std::convert::TryFrom;

//     #[derive(Clone, Ent)]
//     struct TestEnt<T: Clone + TryFrom<Value, Error = &'static str> + Into<Value> + 'static> {
//         #[ent(id)]
//         id: Id,

//         #[ent(database)]
//         database: Option<Box<dyn Database>>,

//         #[ent(created)]
//         created: u64,

//         #[ent(last_updated)]
//         last_updated: u64,

//         #[ent(field)]
//         my_field: T,
//     }
// }
