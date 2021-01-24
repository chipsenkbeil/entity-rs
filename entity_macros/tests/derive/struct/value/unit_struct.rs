use entity::{Primitive, Value, ValueLike};

#[test]
fn instance() {
    #[derive(ValueLike)]
    struct CustomValue;

    assert_eq!(Value::from(CustomValue), Value::Primitive(Primitive::Unit));

    assert!(CustomValue::try_from(Value::Primitive(Primitive::Unit)).is_ok());
}
