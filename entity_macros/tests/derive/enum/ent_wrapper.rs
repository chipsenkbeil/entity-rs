use entity::*;

#[test]
fn implements_ent_wrapper_such_that_all_variant_types_can_be_wrapped() {
    #[simple_ent]
    struct TestEnt1 {}

    #[simple_ent]
    struct TestEnt2 {}

    #[derive(Clone, Ent, EntWrapper)]
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

    #[derive(Clone, Ent, EntWrapper)]
    enum TestEntEnum {
        Variant(TestEnt1),
    }

    let ent: Box<dyn Ent> = Box::new(TestEnt2::build().finish().unwrap());
    let wrapped = <TestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(wrapped.is_none());
}

#[test]
fn supports_wrap_attr_to_mark_variants_that_are_nested_enums() {
    #[simple_ent]
    struct TestEnt1 {}

    #[simple_ent]
    struct TestEnt2 {}

    #[derive(Clone, Ent, EntWrapper)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    #[derive(Clone, Ent, EntWrapper)]
    enum NestedTestEntEnum {
        One(TestEnt1),
        #[ent(wrap)]
        Nested(TestEntEnum),
        Two(TestEnt2),
    }

    // Goes in order, so should succeed with One(...) before Nested(...)
    let ent: Box<dyn Ent> = Box::new(TestEnt1::build().finish().unwrap());
    let wrapped = <NestedTestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, Some(NestedTestEntEnum::One(_))));

    // Goes in order, so should succeed with Nested(...) before Two(...)
    let ent: Box<dyn Ent> = Box::new(TestEnt2::build().finish().unwrap());
    let wrapped = <NestedTestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, Some(NestedTestEntEnum::Nested(_))));
}

#[test]
fn should_return_none_if_trying_to_wrap_a_type_in_a_nested_variant_not_marked_wrap() {
    #[simple_ent]
    struct TestEnt1 {}

    #[simple_ent]
    struct TestEnt2 {}

    #[derive(Clone, Ent, EntWrapper)]
    enum TestEntEnum {
        One(TestEnt1),
        Two(TestEnt2),
    }

    #[derive(Clone, Ent, EntWrapper)]
    enum NestedTestEntEnum {
        Nested(TestEntEnum),
    }

    let ent: Box<dyn Ent> = Box::new(TestEnt1::build().finish().unwrap());
    let wrapped = <NestedTestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, None));

    let ent: Box<dyn Ent> = Box::new(TestEnt2::build().finish().unwrap());
    let wrapped = <NestedTestEntEnum as EntWrapper>::wrap_ent(ent);
    assert!(matches!(wrapped, None));
}
