#![no_implicit_prelude]
// NOTE: This file exists to validate that the prelude can be excluded and the
//       macros produce code with proper pathing; no tests are needed here as
//       this is purely validating that the macros are hygienic via compilation

// NOTE: async-graphql's macros seem to generate async_graphql::... instead
//       of ::async_graphql::...; so, we need to import this for now.
use ::async_graphql;

// NOTE: async-trait does not provide the full path to Box<...>, so we must
//       manually import it for now
//
//       https://github.com/dtolnay/async-trait/issues/163
use ::std::boxed::Box;

#[::entity::simple_ent]
#[derive(::entity_async_graphql_macros::EntObject, ::entity_async_graphql_macros::EntFilter)]
struct Person {
    name: ::std::string::String,
    age: ::std::primitive::u8,

    #[ent(edge)]
    address: Address,
}

#[::entity::simple_ent]
#[derive(::entity_async_graphql_macros::EntObject, ::entity_async_graphql_macros::EntFilter)]
struct Address {
    street: ::std::string::String,
    city: ::std::string::String,
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
