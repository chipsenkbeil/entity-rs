# entity-rs: Library & macros for entity data structures

[![Build Status][build_img]][build_lnk]
[![Crates.io][crates_img]][crates_lnk]
[![Docs.rs][doc_img]][doc_lnk]
[![entity: rustc 1.49+]][Rust 1.49]
[![entity_macros: rustc 1.49+]][Rust 1.49]

[build_img]: https://github.com/chipsenkbeil/entity-rs/workflows/CI/badge.svg
[build_lnk]: https://github.com/chipsenkbeil/entity-rs/actions
[crates_img]: https://img.shields.io/crates/v/entity.svg
[crates_lnk]: https://crates.io/crates/entity
[doc_img]: https://docs.rs/entity/badge.svg
[doc_lnk]: https://docs.rs/entity
[entity: rustc 1.49+]: https://img.shields.io/badge/entity-rustc_1.49+-lightgray.svg
[entity_macros: rustc 1.49+]:
https://img.shields.io/badge/entity_macros-rustc_1.49+-lightgray.svg
[Rust 1.49]: https://blog.rust-lang.org/2020/12/31/Rust-1.49.0.html

A simplistic framework based on [TAO, Facebook's distributed database for Social Graph](https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf).

Requires Rust 1.49+.

## Getting Started

### Installation

Import **Entity** into your project by adding the following line to your
Cargo.toml. `entity_macros` contains the macros needed to derive and/or
transform your data to be compatible with supported databases and queries.

```toml
[dependencies]
entity = "0.3.0"
```

For most use cases, you will want to also import the macros, which will bring
in the `entity_macros` crate:

```toml
[dependencies]
entity = { version = "0.3.0", features = ["macros"] }
```

### Defining data using macros

The following is an example of defining a `User` data structure that contains
an _age_ and _name_ field as well as references to many other `User` instances
under the edge name _friends_.

```rust
use entity::simple_ent;

#[simple_ent]
struct User {
    name: String,
    age: u8,

    #[ent(edge)]
    friends: Vec<User>,
}
```

This generates a variety of trait implementations and additional data
structures to work with the `User` struct against any database.

#### Loading an ent by id

```rust
use entity::EntLoader;

let db = /* entity::WeakDatabaseRc instance */;

// Loads the user from the provided database reference
let user = User::load_from_db_strict(db, 123).expect("Database available");

// Loads the user from the globally-set database
let user = User::load_strict(123).expect("Database available");
```

#### Accessing ent fields

Every object implementing the `Ent` trait is able to access a variety of
common data including abstract field information:

```rust
use entity::{Ent, Primitive, Value};

let user = /* User instance */;

// Access a list of all field names
assert_eq!(user.field_names(), vec![String::from("name"), String::from("age")]);

// Access individual field values, which are exposed using entity's
// generic Value enum
assert_eq!(user.field("name"), Some(Value::Text(/* ... */)));
assert_eq!(user.field("age"), Some(Value::Primitive(Primitive::Number(/* ... */))));
```

When using macros to generate an ent, typed accessors are also provided as
seen below:

```rust
let user = /* User instance */;

// Accesses fields and returns a reference to their actual type NOT
// wrapped in entity's generic Value enum
assert_eq!(user.get_name(), &String::from("..."));
assert_eq!(user.get_age(), &123);
```

#### Accessing ent edges

Every object implementing the `Ent` trait is able to access a variety of
abstract edge information:

```rust
use entity::{Ent, EdgeValue};

let user = /* User instance */;

// Access a list of all edge names
assert_eq!(user.edge_names(), vec![String::from("friends")]);

// Access specific edge information (does not load it)
assert_eq!(user.edge("friends"), Some(EdgeValue::Many(vec![124, 125, /* ... */])));

// Load an edge by name, returning a Vec<Box<dyn Ent>>
let friends: Vec<Box<dyn Ent>> = user.load_edge("friends").expect("Database available");
```

When using macros to generate an ent, typed edge accessors are also provided
as seen below:

```rust
let user = /* User instance */;

// Access the ids of ents referenced by the edge
assert_eq!(user.my_friends_ids(), vec![124, 125, /* ... */]);

// Load the ents referenced by the edge into memory
let friends: Vec<User> = user.load_friends().expect("Database available");
```

#### Querying an ent

Alongside loading ents by their ids, the full suite that `entity-rs` provides
also includes the ability to query arbitrary data structures using a concept of
queries and associated predicates:

```rust
use entity::{EntQuery, Predicate as P, Query};

let db = /* WeakDatabaseRc instance */;

// Produce a new query to search for ents with an age field that is 18 or higher
let ents: Vec<Box<dyn Ent>> = Query::default()
  .where_field("age", P::greater_than_or_equals(18))
  .execute_with_db(db)
  .expect("Database available");

// If the global database has been configured, can also be used in this manner
let ents: Vec<Box<dyn Ent>> = Query::default()
  .where_field("age", P::greater_than_or_equals(18))
  .execute()
  .expect("Database available");
```

When using macros to generate an ent, a companion query struct that
provides stricter types on queries is also created using the name
`{Ent}Query`:

```rust
use entity::{EntQuery, TypedPredicate as P};

let db = /* WeakDatabaseRc instance */;

// Produce a new query to search for ents with an age field that is 18 or higher
let users: Vec<User> = User::query()
  .where_age(P::greater_than_or_equals(18))
  .execute_with_db(db)
  .expect("Database available");

// If the global database has been configured, can also be used in this manner
let users: Vec<User> = User::query()
  .where_age(P::greater_than_or_equals(18))
  .execute()
  .expect("Database available");
```

### Examples

* [`async-graphql`](integrations/entity-async-graphql/examples/user.rs):
  example of using `entity-rs` with `async-graphql`
* [`inmemory`](integrations/entity-inmemory/examples/user.rs): example of using
  `entity-rs` with a custom inmemory database
* [`sled`](integrations/entity-sled/examples/user.rs): example of using
  `entity-rs` with `sled`

## Feature Flags

Entity provides a few feature flags:

* **`global`** - Enables use of a database stored as a global variable,
  providing shortcuts in creating and retrieving ents.
* **`macros`** - Enables macros for deriving ents and exposing a cleaner
  declarative API for ents. (Imports `entity_macros` directly)
* **`serde-1`** - Provides serde serialization module and associated functionality for ents
  through the use of [typetag](https://github.com/dtolnay/typetag). This will
  require that all ents implement [Serialize](https://docs.serde.rs/serde/trait.Serialize.html)
  and [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html).
  * Requires `serde` and `typetag` to be included in dependencies.
