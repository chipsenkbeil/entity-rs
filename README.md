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
entity = "0.2"
```

For most use cases, you can import all features using the `full` flag, or for
a more tailored experience can import individual features:

```toml
[dependencies]
entity = { version = "0.2", features = ["global", "macros", "inmemory_db"] }
```

### Example of defining data

```rust
use entity::*;

#[simple_ent]
struct User {
    name: String,
    age: u8,

    #[ent(edge)]
    friends: Vec<User>,
}
```

## Feature Flags

Entity provides a few feature flags:

* **`full`** - Enables all features.
* **`global`** - Enables use of a database stored as a global variable,
  providing shortcuts in creating and retrieving ents.
* **`macros`** - Enables macros for deriving ents and exposing a cleaner
  declarative API for ents. (Imports `entity_macros` directly)
* **`inmemory_db`** - Enables the in-memory database for use with ent objects.
  This does not bring in `serde-1` by default, but including that feature will
  also support persisting the database to the filesystem.
* **`sled_db`** - Enables the sled database for use with ent objects. Because
  of the nature of sled, this will also pull in `serde-1`.
* **`serde-1`** - Provides serde serialization module and associated functionality for ents
  through the use of [typetag](https://github.com/dtolnay/typetag). This will
  require that all ents implement [Serialize](https://docs.serde.rs/serde/trait.Serialize.html)
  and [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html).
  * Requires `serde` and `typetag` to be included in dependencies.
