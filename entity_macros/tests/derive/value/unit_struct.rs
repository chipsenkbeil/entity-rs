use entity::{PrimitiveValue, Value};
use std::convert::TryFrom;

#[test]
fn instance() {
    #[derive(Value)]
    struct CustomValue;

    assert_eq!(
        Value::from(CustomValue),
        Value::Primitive(PrimitiveValue::Unit)
    );

    assert!(CustomValue::try_from(Value::Primitive(PrimitiveValue::Unit)).is_ok());
}
