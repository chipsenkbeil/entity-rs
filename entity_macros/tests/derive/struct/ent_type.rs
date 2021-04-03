use entity::*;

#[test]
fn implements_as_concrete_ent_type() {
    #[derive(Clone, EntType)]
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

    assert_eq!(
        TestEnt::type_data(),
        EntTypeData::Concrete {
            ty: concat!(module_path!(), "::TestEnt")
        }
    );
}
