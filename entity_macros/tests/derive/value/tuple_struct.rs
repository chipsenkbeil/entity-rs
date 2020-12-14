use entity::Value;
use std::convert::TryFrom;

#[test]
fn no_fields() {
    #[derive(Value)]
    struct CustomValue();

    assert_eq!(Value::from(CustomValue()), Value::List(vec![]));
    assert!(CustomValue::try_from(Value::List(vec![])).is_ok());
    assert!(CustomValue::try_from(Value::List(vec![Value::from(1)])).is_err());
}

#[test]
fn one_field() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue(u32);

    assert_eq!(
        Value::from(CustomValue(3)),
        Value::List(vec![Value::from(3u32)])
    );
    assert!(CustomValue::try_from(Value::List(vec![])).is_err());
    assert_eq!(
        CustomValue::try_from(Value::List(vec![Value::from(1u32)])).unwrap(),
        CustomValue(1),
    );
    assert!(
        CustomValue::try_from(Value::List(vec![Value::from(1u32), Value::from(2u32)])).is_err()
    );
}

#[test]
fn multiple_fields_of_same_type() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue(u32, u32);

    assert_eq!(
        Value::from(CustomValue(3, 8)),
        Value::List(vec![Value::from(3u32), Value::from(8u32)])
    );
    assert!(CustomValue::try_from(Value::List(vec![])).is_err());
    assert_eq!(
        CustomValue::try_from(Value::List(vec![Value::from(1u32), Value::from(2u32)])).unwrap(),
        CustomValue(1, 2),
    );
    assert!(CustomValue::try_from(Value::List(vec![
        Value::from(1u32),
        Value::from(2u32),
        Value::from(3u32)
    ]))
    .is_err());
}

#[test]
fn multiple_fields_of_different_types() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue(u32, String);

    assert_eq!(
        Value::from(CustomValue(3, String::from("test"))),
        Value::List(vec![Value::from(3u32), Value::from("test")])
    );
    assert!(CustomValue::try_from(Value::List(vec![])).is_err());
    assert_eq!(
        CustomValue::try_from(Value::List(vec![Value::from(1u32), Value::from("test")])).unwrap(),
        CustomValue(1, String::from("test")),
    );
    assert!(CustomValue::try_from(Value::List(vec![
        Value::from(1u32),
        Value::from("test"),
        Value::from(3u32)
    ]))
    .is_err());
}
