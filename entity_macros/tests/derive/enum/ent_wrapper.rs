use entity::*;

#[test]
fn implements_ent_wrapper_such_that_all_variant_types_can_be_wrapped() {
    #[simple_ent]
    struct TestEnt1 {}

    #[simple_ent]
    struct TestEnt2 {}

    #[derive(Clone, Ent)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    let ent: Box<dyn Ent> = Box::new(TestEnt1::build().finish().unwrap());
    let wrapped = <TestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, Some(TestEntEnum::One(_))));

    let ent: Box<dyn Ent> = Box::new(TestEnt2::build().finish().unwrap());
    let wrapped = <TestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, Some(TestEntEnum::Two(_))));
}

#[test]
fn implements_ent_wrapper_such_that_type_not_in_variants_cannot_be_wrapped() {
    #[simple_ent]
    struct TestEnt1 {}

    #[simple_ent]
    struct TestEnt2 {}

    #[derive(Clone, Ent)]
    enum TestEntEnum {
        Variant(TestEnt1),
    }

    let ent: Box<dyn Ent> = Box::new(TestEnt2::build().finish().unwrap());
    let wrapped = <TestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(wrapped.is_none());
}
