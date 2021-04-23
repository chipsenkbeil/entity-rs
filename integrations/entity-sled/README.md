# entity-sled

Provides a wrapper database around [`sled`](https://github.com/spacejam/sled)
to support and maintain `entity` objects.

## Example

```rust
use entity_sled::SledDatabase;

// Make our temporary sled::db
let config = sled::Config::new().temporary(true);
let db = config.open().expect("Database created successfully");

// Define our wrapper (SledDatabase) around a tradition sled::db
let db = SledDatabase::new(db);
```

## Special Notes

Requires that `entity` have the `serde-1` flag enabled as all objects must be
serializable & deserializable as well as support `typetag`.
