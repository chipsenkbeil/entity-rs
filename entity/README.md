# entity-rs: Library & macros for entity data structures

[![Build Status][build_img]][build_lnk] [![Crates.io][crates_img]][crates_lnk] [![Docs.rs][doc_img]][doc_lnk]

[build_img]: https://github.com/chipsenkbeil/entity-rs/workflows/CI/badge.svg
[build_lnk]: https://github.com/chipsenkbeil/entity-rs/actions
[crates_img]: https://img.shields.io/crates/v/entity.svg
[crates_lnk]: https://crates.io/crates/entity
[doc_img]: https://docs.rs/entity/badge.svg
[doc_lnk]: https://docs.rs/entity

A simplistic framework for connected data structures modeled after
[Facebook's social graph API, TAO](https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf).

## Getting Started

### Installation

Import **Entity** into your project by adding the following line to your
Cargo.toml. `entity_macros` contains the macros needed to derive and/or
transform your data to be compatible with supported databases and queries.

```toml
[dependencies]
entity = "0.1"
```

Several features come out-of-the-box such as the inmemory database and macros
used for more concise entity creation. See the feature flag section for more
details.

### Example

```rust
use entity::*;

#[simple_ent]
struct User {
    name: String,
    age: u8,

    #[ent(edge(type = "User"))]
    friends: Vec<Id>,
}

let db = InmemoryDatabase::default();
let alice = UserBuilder::default()
    .name(String::from("Alice"))
    .age(30)
    .friends(Vec::new())
    .build()
    .unwrap();
let bob = UserBuilder::default()
    .name(String::from("Bob"))
    .age(35)
    .friends(Vec::new())
    .build()
    .unwrap();
let carol = UserBuilder::default()
    .name(String::from("Carol"))
    .age(27)
    .friends(Vec::new())
    .build()
    .unwrap();
let dan = UserBuilder::default()
    .name(String::from("Dan"))
    .age(25)
    .friends(Vec::new())
    .build()
    .unwrap();
```

### Databases

**Entity** utilizes a simplistic [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete)
database API to manage instances of entity objects.

Out of the box, **Entity** provides an in-memory database available that can
be used for small-scale usage of objects. This is powered by a [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
and [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) to be thread-safe.

```rust
use entity::*;

let db = InmemoryDatabase::default();
```

Additionally, **Entity** can support [sled](https://github.com/spacejam/sled)
for a lightweight, transactional database by adding the feature `sled_db`.

```rust
use entity::*;

let db = SledDatabase::new(
    // Requires importing sled yourself as this API wraps around a sled::Db
    sled::open("my_db").unwrap()
);
```

If you would prefer to implement your own database wrapper, you can implement
the [Database](https://docs.rs/entity/*/entity/trait.Database.html) trait.

```rust
use entity::*;

// Database must be cloneable and should not be expensive to do so. For
// instance, a database struct could contain fields that are wrapped in
// Arc to make them thread safe and cheap to clone as a reference is maintained
#[derive(Clone)]
pub struct MyDatabase;

impl Database for MyDatabase {
    /// Retrieves a copy of a single, generic ent with the corresponding id
    fn get(&self, id: Id) -> DatabaseResult<Option<Box<dyn Ent>>> {
        todo!()
    }

    /// Removes the ent with the corresponding id, triggering edge
    /// processing for all disconnected ents. Returns a boolean indicating
    /// if an ent was removed.
    fn remove(&self, id: Id) -> DatabaseResult<bool> {
        todo!()
    }

    /// Inserts a new ent using its id as the primary index, overwriting
    /// any ent with a matching id. If the ent's id is set to the ephemeral
    /// id (of 0), a unique id will be assigned to the ent prior to being
    /// inserted.
    ///
    /// The ent's id is returned after being inserted.
    fn insert(&self, ent: Box<dyn Ent>) -> DatabaseResult<Id> {
        todo!()
    }

    /// Performs a retrieval of multiple ents of any type
    fn get_all(&self, ids: Vec<Id>) -> DatabaseResult<Vec<Box<dyn Ent>>> {
        todo!()
    }

    /// Finds all generic ents that match the query
    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Box<dyn Ent>>> {
        todo!()
    }
}
```

### Defining Data Structures

At the core of **Entity** is defining your data structures. Out of the box,
this library provides one pre-defined structure that can hold arbitrary fields
and edges, [UntypedEnt](https://docs.rs/entity/*/entity/struct.UntypedEnt.html). This
implements the rather lengthy [Ent](https://docs.rs/entity/*/entity/trait.Ent.html)
trait that is the backbone of the **Entity** framework.

```rust
use entity::*;

let ent = UntypedEnt::from_collections(
    // An id unique to the ent; providing 0 as the id will have it replaced
    // with a unique id upon being saved in a database
    123,

    // Fields associated with the ent instance (like a struct field)
    vec![
        Field::new("field1", 456),
        Field::new("field2", "some text"),
    ],

    // Edges to other ents by their ids
    vec![
        Edge::new("edge1", 456),
    ],
);
```

Normally, you would prefer to implement your own strongly-typed data structure
instead of using the dynamic one above. To do this, you have three options:

1. Implement the [Ent](https://docs.rs/entity/*/entity/trait.Ent.html) trait
   for your struct
2. Derive an implementation using the special macro available from `entity_macros`
   or via the feature `macros` on the `entity` crate
3. Transform a struct using the `simple_ent` attribute macro available from
   `entity_macros` or via the feature `macros` on the `entity` create

For the below example, we'll assume that you have added the `macros` feature
to the `entity` crate.

```rust
use entity::{Ent, Id, Database, simple_ent};

// All entities must be cloneable
#[derive(Clone, Ent)]
pub struct MyEnt {
    // One field in the ent must be marked as its id
    #[ent(id)]
    id: Id,

    // One field in the ent must be marked as its database
    // reference, which is used for loading/saving/refreshing
    #[ent(database)]
    database: Option<Box<dyn Database>>,

    // One field must be marked as the ent's creation timestamp
    #[ent(created)]
    created: u64,

    // One field must be marked as the ent's last updated timestamp
    #[ent(last_updated)]
    last_updated: u64,

    // Any field that belongs as data to this ent is marked as such
    #[ent(field)]
    field1: u32,

    #[ent(field)]
    field2: String,

    // Any association to other ents is marked as an edge with the
    // struct field being an Option<Id>, Id, or Vec<Id>
    #[ent(edge(type = "MyEnt"))]
    maybe_mine: Option<Id>,

    #[ent(edge(type = "SomeOtherEnt"))]
    other: Id,
}

// The simple_ent attribute macro will modify a struct by injecting the
// needed fields and deriving Clone and/or Ent where needed
#[simple_ent]
pub struct SomeOtherEnt {
    #[ent(field)]
    my_field: f64,
}
```

### Queries

TODO

### Deriving

*Requires `entity_macros` or enabling the `macros` feature for
`entity`.*

TODO

## Feature Flags

Entity provides a few feature flags:

* **`inmemory_db`** *(enabled by default)* - Enables the in-memory database for use with ent objects.
  This does not bring in `serde-1` by default, but including that feature will
  also support persisting the database to the filesystem.
* **`sled_db`** - Enables the sled database for use with ent objects. Because
  of the nature of sled, this will also pull in `serde-1`.
* **`serde-1`** - Provides serde serialization module and associated functionality for ents
  through the use of [typetag](https://github.com/dtolnay/typetag). This will
  require that all ents implement [Serialize](https://docs.serde.rs/serde/trait.Serialize.html)
  and [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html).
* **`macros`** *(enabled by default)* - Importing macros from `entity_macros` directly from **entity**.
