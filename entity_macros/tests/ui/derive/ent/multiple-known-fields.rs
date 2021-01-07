//! Test that trying to declare multiple ent-fields on a single field fails properly.
//!
//! This test **must not** claim that `TestEnt` is missing any required `ent` fields;
//! if it does, that likely indicates a bug where field analysis is bailing out too early.

use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id, database)]
    id: Id,

    #[ent(database, id)]
    database: WeakDatabaseRc,

    #[ent(created, last_updated)]
    created: u64,

    #[ent(last_updated, id)]
    last_updated: u64,
}

fn main() {}