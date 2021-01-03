#![no_implicit_prelude]
// NOTE: This file exists to validate that the prelude can be excluded and the
//       macros produce code with proper pathing; no tests are needed here as
//       this is purely validating that the macros are hygienic via compilation

#[::entity::simple_ent]
struct TestEnt1 {
    field1: ::std::string::String,
    field2: ::std::primitive::u64,

    #[ent(edge(type = "TestEnt2", policy = "nothing"))]
    edge1: ::std::option::Option<::entity::Id>,
    #[ent(edge(type = "TestEnt2", policy = "shallow"))]
    edge2: ::entity::Id,
    #[ent(edge(type = "TestEnt2", policy = "deep"))]
    edge3: ::std::vec::Vec<::entity::Id>,

    #[ent(edge(type = "TestEnt3", wrap))]
    edge4: ::std::option::Option<::entity::Id>,
    #[ent(edge(type = "TestEnt3", wrap))]
    edge5: ::entity::Id,
    #[ent(edge(type = "TestEnt3", wrap))]
    edge6: ::std::vec::Vec<::entity::Id>,
}

#[::entity::simple_ent]
struct TestEnt2 {}

#[::entity::simple_ent]
enum TestEnt3 {
    One(TestEnt1),
    Two(TestEnt2),
}

#[derive(::std::clone::Clone, ::entity::Value)]
struct TestValue1;

#[derive(::std::clone::Clone, ::entity::Value)]
struct TestValue2(::std::primitive::u64);

#[derive(::std::clone::Clone, ::entity::Value)]
struct TestValue3 {
    a: ::std::primitive::u64,
}
