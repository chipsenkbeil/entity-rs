//! This test ensures that all errors with a given macro invocation are returned
//! at once, as opposed to returning after the first error.

use entity::{Ent, Id, WeakDatabaseRc};

#[derive(Clone, Ent)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(created)]
    created2: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(last_updated)]
    last_updated2: u64,
}

fn main() {}
