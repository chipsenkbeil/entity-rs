use entity::{PrimitiveValue, Value};
use std::{collections::HashMap, convert::TryFrom};

#[test]
fn no_fields() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue {}

    assert_eq!(Value::from(CustomValue {}), Value::Map(HashMap::new()));
    assert!(CustomValue::try_from(Value::Map(HashMap::new())).is_ok());

    // Will ignore extra fields
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("test"), Value::from(3u32));
            map
        }))
        .unwrap(),
        CustomValue {},
    );
}

#[test]
fn one_field() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue {
        a: u32,
    }

    assert_eq!(
        Value::from(CustomValue { a: 3 }),
        Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map
        })
    );

    assert!(CustomValue::try_from(Value::Map(HashMap::new())).is_err());
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map
        }))
        .unwrap(),
        CustomValue { a: 3 }
    );

    // Will fail if field has wrong type
    assert!(CustomValue::try_from(Value::Map({
        let mut map = HashMap::new();
        map.insert(String::from("a"), Value::from("text"));
        map
    }))
    .is_err());

    // Will ignore extra fields
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map.insert(String::from("b"), Value::from(3u32));
            map
        }))
        .unwrap(),
        CustomValue { a: 3 }
    );
}

#[test]
fn multiple_fields_of_same_type() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue {
        a: u32,
        b: u32,
    }

    assert_eq!(
        Value::from(CustomValue { a: 3, b: 5 }),
        Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map.insert(String::from("b"), Value::from(5u32));
            map
        })
    );

    assert!(CustomValue::try_from(Value::Map(HashMap::new())).is_err());
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map.insert(String::from("b"), Value::from(5u32));
            map
        }))
        .unwrap(),
        CustomValue { a: 3, b: 5 }
    );
    assert!(CustomValue::try_from(Value::Map({
        let mut map = HashMap::new();
        map.insert(String::from("a"), Value::from(3u32));
        map.insert(String::from("c"), Value::from(3u32));
        map
    }))
    .is_err());
}

#[test]
fn multiple_fields_of_different_types() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue {
        a: u32,
        b: String,
    }

    assert_eq!(
        Value::from(CustomValue {
            a: 3,
            b: String::from("test")
        }),
        Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map.insert(String::from("b"), Value::from("test"));
            map
        })
    );

    assert!(CustomValue::try_from(Value::Map(HashMap::new())).is_err());
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(3u32));
            map.insert(String::from("b"), Value::from("test"));
            map
        }))
        .unwrap(),
        CustomValue {
            a: 3,
            b: String::from("test")
        }
    );
    assert!(CustomValue::try_from(Value::Map({
        let mut map = HashMap::new();
        map.insert(String::from("a"), Value::from(3u32));
        map.insert(String::from("b"), Value::from(3u32));
        map
    }))
    .is_err());
}

#[test]
fn fields_that_also_derives_value() {
    #[derive(Debug, PartialEq, Eq, Value)]
    struct A(u32);

    #[derive(Debug, PartialEq, Eq, Value)]
    struct B {
        inner: String,
    }

    #[derive(Debug, PartialEq, Eq, Value)]
    struct C;

    #[derive(Debug, PartialEq, Eq, Value)]
    struct CustomValue {
        a: A,
        b: B,
        c: C,
    }

    assert_eq!(
        Value::from(CustomValue {
            a: A(3),
            b: B {
                inner: String::from("test")
            },
            c: C
        }),
        Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(vec![3u32]));
            map.insert(
                String::from("b"),
                Value::from({
                    let mut map = HashMap::new();
                    map.insert(String::from("inner"), Value::from("test"));
                    map
                }),
            );
            map.insert(String::from("c"), Value::from(PrimitiveValue::Unit));
            map
        })
    );

    assert!(CustomValue::try_from(Value::Map(HashMap::new())).is_err());
    assert_eq!(
        CustomValue::try_from(Value::Map({
            let mut map = HashMap::new();
            map.insert(String::from("a"), Value::from(vec![3u32]));
            map.insert(
                String::from("b"),
                Value::from({
                    let mut map = HashMap::new();
                    map.insert(String::from("inner"), Value::from("test"));
                    map
                }),
            );
            map.insert(String::from("c"), Value::from(PrimitiveValue::Unit));
            map
        }))
        .unwrap(),
        CustomValue {
            a: A(3),
            b: B {
                inner: String::from("test")
            },
            c: C,
        }
    );
}
