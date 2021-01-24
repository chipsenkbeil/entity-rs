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

#[derive(::std::clone::Clone, ::entity::ValueLike)]
struct TestValue1;

#[derive(::std::clone::Clone, ::entity::ValueLike)]
struct TestValue2(::std::primitive::u64);

#[derive(::std::clone::Clone, ::entity::ValueLike)]
struct TestValue3 {
    a: ::std::primitive::u64,
}

// These traits exist to make sure we properly import using
// ::std::primitive::<TYPE> instead of purely <TYPE>
//
// Only works for Rust 1.43.0+, but our requirement is 1.45.0+ for macros, so
// this is fine.
#[allow(non_camel_case_types)]
trait bool {}
#[allow(non_camel_case_types)]
trait char {}
#[allow(non_camel_case_types)]
trait f32 {}
#[allow(non_camel_case_types)]
trait f64 {}
#[allow(non_camel_case_types)]
trait i128 {}
#[allow(non_camel_case_types)]
trait i16 {}
#[allow(non_camel_case_types)]
trait i32 {}
#[allow(non_camel_case_types)]
trait i64 {}
#[allow(non_camel_case_types)]
trait i8 {}
#[allow(non_camel_case_types)]
trait isize {}
#[allow(non_camel_case_types)]
trait str {}
#[allow(non_camel_case_types)]
trait u128 {}
#[allow(non_camel_case_types)]
trait u16 {}
#[allow(non_camel_case_types)]
trait u32 {}
#[allow(non_camel_case_types)]
trait u64 {}
#[allow(non_camel_case_types)]
trait u8 {}
#[allow(non_camel_case_types)]
trait usize {}
