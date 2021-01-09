use entity::*;

#[test]
fn produces_load_methods_that_pull_an_ent_out_of_a_database() {
    #[derive(Clone, Ent, EntLoader)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    entity::global::with_db(InmemoryDatabase::default(), || {
        let _ = WeakDatabaseRc::upgrade(&entity::global::db())
            .unwrap()
            .insert(Box::new(TestEnt {
                id: 123,
                database: WeakDatabaseRc::new(),
                created: 0,
                last_updated: 0,
            }))
            .unwrap();

        assert!(TestEnt::load(123).unwrap().is_some());
        assert!(TestEnt::load_strict(123).is_ok());
        assert!(TestEnt::load_from_db(entity::global::db(), 123)
            .unwrap()
            .is_some());
        assert!(TestEnt::load_from_db_strict(entity::global::db(), 123).is_ok());

        assert!(TestEnt::load(999).unwrap().is_none());
        assert!(TestEnt::load_strict(999).is_err());
        assert!(TestEnt::load_from_db(entity::global::db(), 999)
            .unwrap()
            .is_none());
        assert!(TestEnt::load_from_db_strict(entity::global::db(), 999).is_err());
    });
}
