use entity::*;

#[test]
fn implements_as_wrapper_type() {
    #[derive(Clone, EntType)]
    struct TestEnt1 {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: WeakDatabaseRc,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,
    }

    #[derive(Clone, EntType)]
    enum TestEnt2 {
        One(TestEnt1),
    }

    assert_eq!(
        TestEnt2::type_data(),
        EntTypeData::Wrapper {
            ty: concat!(module_path!(), "::TestEnt2"),
            wrapped_tys: vec![TestEnt1::type_str()].into_iter().collect(),
        }
    );

    #[derive(Clone, EntType)]
    enum TestEnt3 {
        One(TestEnt1),

        #[ent(wrap)]
        Two(TestEnt2),
    }

    assert_eq!(
        TestEnt3::type_data(),
        EntTypeData::Wrapper {
            ty: concat!(module_path!(), "::TestEnt3"),
            wrapped_tys: vec![TestEnt1::type_str()].into_iter().collect(),
        }
    );
}
