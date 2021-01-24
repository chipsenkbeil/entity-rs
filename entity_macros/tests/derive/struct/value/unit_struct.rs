use entity::{Primitive, Value, ValueLike};

#[test]
fn instance() {
    #[derive(ValueLike)]
    struct CustomValue;

    assert_eq!(
        ValueLike::into_value(CustomValue),
        Value::Primitive(Primitive::Unit)
    );

    assert!(CustomValue::try_from_value(Value::Primitive(Primitive::Unit)).is_ok());
}
