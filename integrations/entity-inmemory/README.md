# entity-inmemory

Provides a custom inmemory database on top of `entity` that leverages a mixture
of [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
to maintain the entities.

## Example

```rust
use entity_inmemory::InmemoryDatabase;

let db = InmemoryDatabase::default();
```

## Feature Flags

Entity provides a few feature flags:

* **`serde-1`** - Provides serde serialization module and associated
  functionality for the database. Ents are supported through the use of
  [typetag](https://github.com/dtolnay/typetag). This will require that all
  ents implement [Serialize](https://docs.serde.rs/serde/trait.Serialize.html)
  and [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html).
  * Requires `serde-1` be enabled on `entity` crate
  * Requires `serde` and `typetag` to be included in dependencies
