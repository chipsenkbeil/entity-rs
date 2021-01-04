#![cfg_attr(not(feature = "global"), allow(unused_imports, unused_variables))]

use crate::{Database, DatabaseRc, WeakDatabaseRc};
use std::sync::Mutex;

#[cfg(feature = "global")]
lazy_static::lazy_static! {
    static ref DATABASE: Mutex<Option<DatabaseRc>> = Mutex::new(None);
    static ref WITH_LOCK: Mutex<()> = Mutex::new(());
}

/// Executes the given function with the provided database as the new, global
/// database, destroying the database once the function completes; locks
/// execution of this function, only allowing one call to `with_db` at a time
pub fn with_db_from_rc<F: FnMut() -> R, R>(database: DatabaseRc, mut f: F) -> R {
    #[cfg(feature = "global")]
    let _lock = WITH_LOCK.lock().unwrap();

    set_db_from_rc(database);
    let result = f();
    destroy_db();
    result
}

/// Executes the given function with the provided database as the new, global
/// database, destroying the database once the function completes; locks
/// execution of this function, only allowing one call to `with_db` at a time
#[inline]
pub fn with_db_from_box<F: FnMut() -> R, R>(database: Box<dyn Database>, f: F) -> R {
    with_db_from_rc(DatabaseRc::new(database), f)
}

/// Executes the given function with the provided database as the new, global
/// database, destroying the database once the function completes; locks
/// execution of this function, only allowing one call to `with_db` at a time
#[inline]
pub fn with_db<D: Database + 'static, F: FnMut() -> R, R>(database: D, f: F) -> R {
    with_db_from_box(Box::new(database), f)
}

/// Returns a weak reference to the global database if it is set, otherwise
/// will return a weak reference that will resolve to None when upgrading
#[inline]
pub fn db() -> WeakDatabaseRc {
    #[cfg(feature = "global")]
    let x = match DATABASE.lock().unwrap().as_ref() {
        Some(x) => DatabaseRc::downgrade(x),
        None => WeakDatabaseRc::new(),
    };

    #[cfg(not(feature = "global"))]
    let x = WeakDatabaseRc::new();

    x
}

/// Sets the global database to the specific database implementation
#[inline]
pub fn set_db<D: Database + 'static>(database: D) -> WeakDatabaseRc {
    set_db_from_box(Box::new(database))
}

/// Sets the global database to the database trait object
#[inline]
pub fn set_db_from_box(database: Box<dyn Database>) -> WeakDatabaseRc {
    set_db_from_rc(DatabaseRc::new(database))
}

/// Sets the global database to the strong reference and returns a weak
/// reference to the same database
#[inline]
pub fn set_db_from_rc(database_rc: DatabaseRc) -> WeakDatabaseRc {
    #[cfg(feature = "global")]
    DATABASE.lock().unwrap().replace(database_rc);
    db()
}

/// Returns true if the global database has been assigned
#[inline]
pub fn has_db() -> bool {
    #[cfg(feature = "global")]
    let x = DATABASE.lock().unwrap().is_some();

    #[cfg(not(feature = "global"))]
    let x = false;

    x
}

/// Removes the global database reference
#[inline]
pub fn destroy_db() {
    #[cfg(feature = "global")]
    DATABASE.lock().unwrap().take();
}

#[cfg(all(test, feature = "global"))]
mod tests {
    use super::*;
    use crate::{DatabaseResult, Ent, Id, Query};

    /// Resets database to starting state
    fn reset_db_state() {
        DATABASE.lock().unwrap().take();
    }

    /// NOTE: We have to run all tests that impact the global database in a
    ///       singular test to avoid race conditions in modifying and checking
    ///       global database state from parallel tests. This is to avoid the
    ///       need to run the entire test infra in a single thread, which is
    ///       much slower.
    #[test]
    fn test_runner() {
        fn db_should_return_empty_weak_ref_if_database_not_set() {
            reset_db_state();

            assert!(
                WeakDatabaseRc::ptr_eq(&db(), &WeakDatabaseRc::new()),
                "Returned weak reference unexpectedly pointing to database"
            );
        }
        db_should_return_empty_weak_ref_if_database_not_set();

        fn db_should_return_weak_ref_for_active_database_if_set() {
            reset_db_state();

            set_db(TestDatabase);
            assert!(
                !WeakDatabaseRc::ptr_eq(&db(), &WeakDatabaseRc::new()),
                "Returned weak reference not pointing to database"
            );
        }
        db_should_return_weak_ref_for_active_database_if_set();

        fn set_db_should_update_the_global_database_with_the_given_instance() {
            reset_db_state();

            assert!(
                !WeakDatabaseRc::ptr_eq(&set_db(TestDatabase), &WeakDatabaseRc::new()),
                "Returned weak reference not pointing to database"
            );

            assert!(DATABASE.lock().unwrap().is_some());
        }
        set_db_should_update_the_global_database_with_the_given_instance();

        fn set_db_from_box_should_update_the_global_database_with_the_given_instance() {
            reset_db_state();

            assert!(
                !WeakDatabaseRc::ptr_eq(
                    &set_db_from_box(Box::new(TestDatabase)),
                    &WeakDatabaseRc::new()
                ),
                "Returned weak reference not pointing to database"
            );

            assert!(DATABASE.lock().unwrap().is_some());
        }
        set_db_from_box_should_update_the_global_database_with_the_given_instance();

        fn set_db_from_rc_should_update_the_global_database_with_the_given_instance() {
            reset_db_state();

            assert!(
                !WeakDatabaseRc::ptr_eq(
                    &set_db_from_rc(DatabaseRc::new(Box::new(TestDatabase))),
                    &WeakDatabaseRc::new()
                ),
                "Returned weak reference not pointing to database"
            );

            assert!(DATABASE.lock().unwrap().is_some());
        }
        set_db_from_rc_should_update_the_global_database_with_the_given_instance();

        fn has_db_should_return_false_if_database_not_set() {
            reset_db_state();

            assert!(!has_db(), "Unexpectedly reported having database");
        }
        has_db_should_return_false_if_database_not_set();

        fn has_db_should_return_false_if_database_destroyed() {
            reset_db_state();

            DATABASE
                .lock()
                .unwrap()
                .replace(DatabaseRc::new(Box::new(TestDatabase)));
            destroy_db();

            assert!(!has_db(), "Unexpectedly reported having database");
        }
        has_db_should_return_false_if_database_destroyed();

        fn has_db_should_return_true_if_database_set() {
            reset_db_state();

            set_db(TestDatabase);
            assert!(has_db(), "Unexpectedly reported NOT having database");
        }
        has_db_should_return_true_if_database_set();

        fn destroy_db_should_remove_global_database_if_set() {
            reset_db_state();

            DATABASE
                .lock()
                .unwrap()
                .replace(DatabaseRc::new(Box::new(TestDatabase)));

            destroy_db();
            assert!(
                DATABASE.lock().unwrap().is_none(),
                "Database was not destroyed"
            );
        }
        destroy_db_should_remove_global_database_if_set();

        fn destroy_db_should_do_nothing_if_global_database_is_not_set() {
            reset_db_state();

            destroy_db();
            assert!(
                DATABASE.lock().unwrap().is_none(),
                "Database was not destroyed"
            );
        }
        destroy_db_should_do_nothing_if_global_database_is_not_set();
    }

    /// Represents a test database so we can run the above tests regardless
    /// of whether the inmemory, sled, or other database feature is active
    struct TestDatabase;

    impl Database for TestDatabase {
        fn get(&self, _id: Id) -> DatabaseResult<Option<Box<dyn Ent>>> {
            unimplemented!()
        }

        fn remove(&self, _id: Id) -> DatabaseResult<bool> {
            unimplemented!()
        }

        fn insert(&self, _ent: Box<dyn Ent>) -> DatabaseResult<Id> {
            unimplemented!()
        }

        fn get_all(&self, _ids: Vec<Id>) -> DatabaseResult<Vec<Box<dyn Ent>>> {
            unimplemented!()
        }

        fn find_all(&self, _query: Query) -> DatabaseResult<Vec<Box<dyn Ent>>> {
            unimplemented!()
        }
    }
}
