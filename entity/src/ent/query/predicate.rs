use crate::Value;
use derivative::Derivative;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    marker::PhantomData,
    ops::RangeInclusive,
    rc::Rc,
};

/// Represents an untyped predicate that can be used to inspect a value for
/// some specified condition
#[derive(Clone, Derivative)]
#[derivative(Debug, PartialEq)]
pub enum Predicate {
    /// Will always be true (not same as equals(true))
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Always;
    /// assert_eq!(p.check(&v), true);
    /// ```
    Always,

    /// Will always be false (not same as equals(false))
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Never;
    /// assert_eq!(p.check(&v), false);
    /// ```
    Never,

    /// Will be true if all predicates return true against the checked value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::And(vec![
    ///     Predicate::GreaterThan(Value::from(122)),
    ///     Predicate::LessThan(Value::from(124)),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::And(vec![
    ///     Predicate::GreaterThan(Value::from(122)),
    ///     Predicate::Never,
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    And(Vec<Predicate>),

    /// Will be true if checked value is a collection where any element
    /// satisifies the predicate
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(vec![1, 2, 3]);
    ///
    /// let p = Predicate::Any(Box::new(Predicate::Equals(Value::from(2))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Any(Box::new(Predicate::Equals(Value::from(999))));
    /// assert_eq!(p.check(&v), false);
    /// ```
    ///
    /// Also supports checking values of a map:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::Any(Box::new(Predicate::Equals(Value::from(2))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Any(Box::new(Predicate::Equals(Value::from(999))));
    /// assert_eq!(p.check(&v), false);
    /// ```
    Any(Box<Predicate>),

    /// Will be true if checked value is a collection that contains the
    /// specified value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(vec![1, 2, 3]);
    ///
    /// let p = Predicate::Contains(Value::from(2));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Contains(Value::from(999));
    /// assert_eq!(p.check(&v), false);
    /// ```
    ///
    /// Also supports checking values of a map:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::Contains(Value::from(2));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Contains(Value::from(999));
    /// assert_eq!(p.check(&v), false);
    /// ```
    Contains(Value),

    /// Will be true if checked value is a collection that contains all of
    /// the specified values
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(vec![1, 2, 3]);
    ///
    /// let p = Predicate::ContainsAll(vec![Value::from(2), Value::from(3)]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::ContainsAll(vec![Value::from(2), Value::from(999)]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    ///
    /// Also supports checking values of a map:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::ContainsAll(vec![Value::from(2), Value::from(3)]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::ContainsAll(vec![Value::from(2), Value::from(999)]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    ContainsAll(Vec<Value>),

    /// Will be true if checked value is a collection that contains any of
    /// the specified values
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(vec![1, 2, 3]);
    ///
    /// let p = Predicate::ContainsAny(vec![Value::from(2), Value::from(999)]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::ContainsAny(vec![Value::from(998), Value::from(999)]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    ///
    /// Also supports checking values of a map:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::ContainsAny(vec![Value::from(2), Value::from(999)]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::ContainsAny(vec![Value::from(998), Value::from(999)]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    ContainsAny(Vec<Value>),

    /// Will be true if checked value equals the specified value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Equals(Value::from(123));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Equals(Value::from(456));
    /// assert_eq!(p.check(&v), false);
    /// ```
    Equals(Value),

    /// Will be true if checked value is greater than the specified value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::GreaterThan(Value::from(100));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::GreaterThan(Value::from(123));
    /// assert_eq!(p.check(&v), false);
    ///
    /// let p = Predicate::GreaterThan(Value::from(456));
    /// assert_eq!(p.check(&v), false);
    /// ```
    GreaterThan(Value),

    /// Will be true if checked value is greater than or equals the specified
    /// value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::GreaterThanOrEquals(Value::from(100));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::GreaterThanOrEquals(Value::from(123));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::GreaterThanOrEquals(Value::from(456));
    /// assert_eq!(p.check(&v), false);
    /// ```
    GreaterThanOrEquals(Value),

    /// Will be true if checked value is a map that contains the specified key
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::HasKey(String::from("a"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::HasKey(String::from("d"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    HasKey(String),

    /// Will be true if checked value is a map that contains the specified key
    /// and corresponding value meets the specified predicate
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::collections::HashMap;
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    /// let v = Value::from(map);
    ///
    /// let p = Predicate::HasKeyWhereValue(
    ///     String::from("a"),
    ///     Box::new(Predicate::Equals(Value::from(1))),
    /// );
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::HasKeyWhereValue(
    ///     String::from("b"),
    ///     Box::new(Predicate::Equals(Value::from(1))),
    /// );
    /// assert_eq!(p.check(&v), false);
    ///
    /// let p = Predicate::HasKeyWhereValue(
    ///     String::from("d"),
    ///     Box::new(Predicate::Equals(Value::from(1))),
    /// );
    /// assert_eq!(p.check(&v), false);
    /// ```
    HasKeyWhereValue(String, Box<Predicate>),

    /// Will be true if checked value is within the specified range
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::InRange(Value::from(100)..=Value::from(200));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::InRange(Value::from(200)..=Value::from(300));
    /// assert_eq!(p.check(&v), false);
    /// ```
    InRange(RangeInclusive<Value>),

    /// Will be true if checked value is within the specified set
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::InSet(vec![
    ///     Value::from(122),
    ///     Value::from(123),
    ///     Value::from(124),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::InSet(vec![
    ///     Value::from(222),
    ///     Value::from(223),
    ///     Value::from(224),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), false);
    /// ```
    InSet(HashSet<Value>),

    /// Will be true if checked value optional and is none
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// let v = Value::from(None::<u8>);
    /// let p = Predicate::IsNone;
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(Some(123));
    /// let p = Predicate::IsNone;
    /// assert_eq!(p.check(&v), false);
    /// ```
    IsNone,

    /// Will be true if checked value passes the lambda function by having
    /// it return true
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// use std::rc::Rc;
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Lambda(Rc::new(|v| v == &Value::from(123)));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Lambda(Rc::new(|v| v == &Value::from(456)));
    /// assert_eq!(p.check(&v), false);
    /// ```
    Lambda(#[derivative(Debug = "ignore", PartialEq = "ignore")] Rc<dyn Fn(&Value) -> bool>),

    /// Will be true if checked value is less than the specified value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::LessThan(Value::from(200));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::LessThan(Value::from(123));
    /// assert_eq!(p.check(&v), false);
    ///
    /// let p = Predicate::LessThan(Value::from(100));
    /// assert_eq!(p.check(&v), false);
    /// ```
    LessThan(Value),

    /// Will be true if checked value is less than or equals the specified
    /// value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::LessThanOrEquals(Value::from(200));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::LessThanOrEquals(Value::from(123));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::LessThanOrEquals(Value::from(100));
    /// assert_eq!(p.check(&v), false);
    /// ```
    LessThanOrEquals(Value),

    /// Will be true if checked value is does not pass the specified predicate
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Not(Box::new(Predicate::Equals(Value::from(200))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Not(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), false);
    /// ```
    Not(Box<Predicate>),

    /// Will be true if checked value does not equal the specified value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::NotEquals(Value::from(456));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::NotEquals(Value::from(123));
    /// assert_eq!(p.check(&v), false);
    /// ```
    NotEquals(Value),

    /// Will be true if checked value is not within the specified range
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::NotInRange(Value::from(200)..=Value::from(300));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::NotInRange(Value::from(100)..=Value::from(200));
    /// assert_eq!(p.check(&v), false);
    /// ```
    NotInRange(RangeInclusive<Value>),

    /// Will be true if checked value is not within the specified set
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::NotInSet(vec![
    ///     Value::from(222),
    ///     Value::from(223),
    ///     Value::from(224),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::NotInSet(vec![
    ///     Value::from(122),
    ///     Value::from(123),
    ///     Value::from(124),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), false);
    /// ```
    NotInSet(HashSet<Value>),

    /// Will be true if checked value is not none and passes the specified
    /// predicate
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// let v = Value::from(Some(123));
    /// let p = Predicate::NotNoneAnd(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(123);
    /// let p = Predicate::NotNoneAnd(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(Some(456));
    /// let p = Predicate::NotNoneAnd(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), false);
    ///
    /// let v = Value::from(456);
    /// let p = Predicate::NotNoneAnd(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), false);
    ///
    /// let v = Value::from(None::<u8>);
    /// let p = Predicate::NotNoneAnd(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), false);
    /// ```
    NotNoneAnd(Box<Predicate>),

    /// Will be true if checked value is either none or passes the specified
    /// predicate
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// let v = Value::from(None::<u8>);
    /// let p = Predicate::NoneOr(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(Some(123));
    /// let p = Predicate::NoneOr(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(123);
    /// let p = Predicate::NoneOr(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let v = Value::from(Some(456));
    /// let p = Predicate::NoneOr(Box::new(Predicate::Equals(Value::from(123))));
    /// assert_eq!(p.check(&v), false);
    /// ```
    NoneOr(Box<Predicate>),

    /// Will be true if any predicate returns true against the checked value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Or(vec![
    ///     Predicate::GreaterThan(Value::from(122)),
    ///     Predicate::LessThan(Value::from(124)),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Or(vec![
    ///     Predicate::Equals(Value::from(122)),
    ///     Predicate::Equals(Value::from(123)),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Or(vec![
    ///     Predicate::Equals(Value::from(122)),
    ///     Predicate::Equals(Value::from(124)),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    Or(Vec<Predicate>),

    /// Will be true if checked value is text that is a substring if the
    /// specified string (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("substring"));
    ///
    /// let p = Predicate::TextContainedIn(String::from("my substring of text"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainedIn(String::from("my string of text"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainedIn(String),

    /// Will be true if checked value is text that is a substring if the
    /// specified string (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("substring"));
    ///
    /// let p = Predicate::TextContainedInCaseInsensitive(String::from("MY SUBSTRING OF TEXT"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainedInCaseInsensitive(String::from("my string of text"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainedInCaseInsensitive(String),

    /// Will be true if checked value is text that contains all of the
    /// specified strings as substrings (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextContainsAll(vec![
    ///     String::from("my"),
    ///     String::from("text"),
    ///     String::from("compared"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainsAll(vec![
    ///     String::from("my"),
    ///     String::from("other"),
    ///     String::from("compared"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainsAll(Vec<String>),

    /// Will be true if checked value is text that contains all of the
    /// specified strings as substrings (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextContainsAllCaseInsensitive(vec![
    ///     String::from("MY"),
    ///     String::from("TEXT"),
    ///     String::from("COMPARED"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainsAllCaseInsensitive(vec![
    ///     String::from("my"),
    ///     String::from("other"),
    ///     String::from("compared"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainsAllCaseInsensitive(Vec<String>),

    /// Will be true if checked value is text that contains any of the
    /// specified strings as substrings (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextContainsAny(vec![
    ///     String::from("something"),
    ///     String::from("text"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainsAny(vec![
    ///     String::from("something"),
    ///     String::from("not"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainsAny(Vec<String>),

    /// Will be true if checked value is text that contains any of the
    /// specified strings as substrings (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextContainsAnyCaseInsensitive(vec![
    ///     String::from("SOMETHING"),
    ///     String::from("TEXT"),
    ///     String::from("OTHER"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextContainsAnyCaseInsensitive(vec![
    ///     String::from("something"),
    ///     String::from("not"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextContainsAnyCaseInsensitive(Vec<String>),

    /// Will be true if checked value is text that ends with the specified string
    /// (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextEndsWith(String::from("text"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextEndsWith(String::from("TEXT"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextEndsWith(String),

    /// Will be true if checked value is text that ends with the specified string
    /// (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextEndsWithCaseInsensitive(String::from("TEXT"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextEndsWithCaseInsensitive(String::from("some"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextEndsWithCaseInsensitive(String),

    /// Will be true if checked value is text that ends with any of the
    /// specified strings (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextEndsWithAny(vec![
    ///     String::from("something"),
    ///     String::from("compared"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextEndsWithAny(vec![
    ///     String::from("something"),
    ///     String::from("COMPARED"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextEndsWithAny(Vec<String>),

    /// Will be true if checked value is text that ends with any of the
    /// specified strings (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextEndsWithAnyCaseInsensitive(vec![
    ///     String::from("SOMETHING"),
    ///     String::from("COMPARED"),
    ///     String::from("OTHER"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextEndsWithAnyCaseInsensitive(vec![
    ///     String::from("something"),
    ///     String::from("text"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextEndsWithAnyCaseInsensitive(Vec<String>),

    /// Will be true if checked value is text that equals the specified string
    /// (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextEqualsCaseInsensitive(String::from("SOME TEXT"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextEqualsCaseInsensitive(String::from("other text"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextEqualsCaseInsensitive(String),

    /// Will be true if checked value is text that does not equal the specified
    /// string (case insensitive); or if any other type than text
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextNotEqualsCaseInsensitive(String::from("other text"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextNotEqualsCaseInsensitive(String::from("SOME TEXT"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextNotEqualsCaseInsensitive(String),

    /// Will be true if checked value is text that is found within the
    /// specified set of strings
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from("some text");
    ///
    /// let p = Predicate::TextInSetCaseInsensitive(vec![
    ///     String::from("SOME"),
    ///     String::from("SOME TEXT"),
    ///     String::from("TEXT"),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextInSetCaseInsensitive(vec![
    ///     String::from("OTHER"),
    ///     String::from("OTHER TEXT"),
    ///     String::from("TEXT"),
    /// ].into_iter().collect());
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextInSetCaseInsensitive(HashSet<String>),

    /// Will be true if checked value is text that starts with the specified string
    /// (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextStartsWith(String::from("some"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextStartsWith(String::from("SOME"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextStartsWith(String),

    /// Will be true if checked value is text that starts with the specified string
    /// (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("some text"));
    ///
    /// let p = Predicate::TextStartsWithCaseInsensitive(String::from("SOME"));
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextStartsWithCaseInsensitive(String::from("text"));
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextStartsWithCaseInsensitive(String),

    /// Will be true if checked value is text that starts with any of the
    /// specified strings as substrings (case sensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextStartsWithAny(vec![
    ///     String::from("something"),
    ///     String::from("my"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextStartsWithAny(vec![
    ///     String::from("something"),
    ///     String::from("MY"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextStartsWithAny(Vec<String>),

    /// Will be true if checked value is text that starts with any of the
    /// specified strings as substrings (case insensitive)
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(String::from("my text that is compared"));
    ///
    /// let p = Predicate::TextStartsWithAnyCaseInsensitive(vec![
    ///     String::from("SOMETHING"),
    ///     String::from("MY"),
    ///     String::from("OTHER"),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::TextStartsWithAnyCaseInsensitive(vec![
    ///     String::from("something"),
    ///     String::from("not"),
    ///     String::from("other"),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    TextStartsWithAnyCaseInsensitive(Vec<String>),

    /// Will be true if only one predicate returns true against the checked value
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    /// let v = Value::from(123);
    ///
    /// let p = Predicate::Xor(vec![
    ///     Predicate::Equals(Value::from(122)),
    ///     Predicate::Equals(Value::from(123)),
    /// ]);
    /// assert_eq!(p.check(&v), true);
    ///
    /// let p = Predicate::Xor(vec![
    ///     Predicate::GreaterThan(Value::from(122)),
    ///     Predicate::LessThan(Value::from(124)),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    ///
    /// let p = Predicate::Xor(vec![
    ///     Predicate::Equals(Value::from(122)),
    ///     Predicate::Equals(Value::from(124)),
    /// ]);
    /// assert_eq!(p.check(&v), false);
    /// ```
    Xor(Vec<Predicate>),
}

impl Predicate {
    /// Checks if the predicate is satisfied by the given value
    pub fn check(&self, value: &Value) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::And(list) => list.iter().all(|p| p.check(value)),
            Self::Any(p) => match value {
                Value::List(x) => x.iter().any(|v| p.check(v)),
                Value::Map(x) => x.iter().any(|(_, v)| p.check(v)),
                _ => false,
            },
            Self::Contains(v) => match value {
                Value::List(x) => x.contains(v),
                Value::Map(x) => x.iter().any(|(_, vv)| v == vv),
                _ => false,
            },
            Self::ContainsAll(list) => match value {
                Value::List(x) => list.iter().all(|v| x.contains(v)),
                Value::Map(x) => list.iter().all(|v| x.iter().any(|(_, vv)| v == vv)),
                _ => false,
            },
            Self::ContainsAny(list) => match value {
                Value::List(x) => list.iter().any(|v| x.contains(v)),
                Value::Map(x) => list.iter().any(|v| x.iter().any(|(_, vv)| v == vv)),
                _ => false,
            },
            Self::TextEndsWith(s) => match value {
                Value::Text(t) => t.ends_with(s),
                _ => false,
            },
            Self::TextEndsWithCaseInsensitive(s) => match value {
                Value::Text(t) => t.to_lowercase().ends_with(&s.to_lowercase()),
                _ => false,
            },
            Self::Equals(v) => value == v,
            Self::TextEqualsCaseInsensitive(s) => match value {
                Value::Text(t) => t.to_lowercase() == s.to_lowercase(),
                _ => false,
            },
            Self::GreaterThan(v) => value > v,
            Self::GreaterThanOrEquals(v) => value >= v,
            Self::HasKey(k) => match value {
                Value::Map(x) => x.contains_key(k),
                _ => false,
            },
            Self::HasKeyWhereValue(k, p) => match value {
                Value::Map(x) => x.get(k).map(|v| p.check(v)).unwrap_or_default(),
                _ => false,
            },
            Self::InRange(r) => value >= r.start() && value <= r.end(),
            Self::InSet(v) => v.contains(value),
            Self::TextInSetCaseInsensitive(list) => match value {
                Value::Text(t) => list.iter().any(|s| t.to_lowercase() == s.to_lowercase()),
                _ => false,
            },
            Self::IsNone => matches!(value, Value::Optional(None)),
            Self::Lambda(f) => f(value),
            Self::LessThan(v) => value < v,
            Self::LessThanOrEquals(v) => value <= v,
            Self::Not(p) => !p.check(value),
            Self::NotEquals(v) => value != v,
            Self::TextNotEqualsCaseInsensitive(s) => match value {
                Value::Text(t) => t.to_lowercase() != s.to_lowercase(),
                _ => false,
            },
            Self::NotInRange(r) => value < r.start() || value > r.end(),
            Self::NotInSet(list) => !list.contains(value),
            Self::NotNoneAnd(p) => match value {
                Value::Optional(Some(v)) => p.check(v),
                v => p.check(v),
            },
            Self::NoneOr(p) => match value {
                Value::Optional(None) => true,
                v => p.check(v),
            },
            Self::Or(list) => list.iter().any(|p| p.check(value)),
            Self::TextStartsWith(s) => match value {
                Value::Text(t) => t.starts_with(s),
                _ => false,
            },
            Self::TextStartsWithCaseInsensitive(s) => match value {
                Value::Text(t) => t.to_lowercase().starts_with(&s.to_lowercase()),
                _ => false,
            },
            Self::TextContainedIn(s) => match value {
                Value::Text(t) => s.contains(t),
                _ => false,
            },
            Self::TextContainedInCaseInsensitive(s) => match value {
                Value::Text(t) => s.to_lowercase().contains(&t.to_lowercase()),
                _ => false,
            },
            Self::TextContainsAll(list) => match value {
                Value::Text(t) => list.iter().all(|s| t.contains(s)),
                _ => false,
            },
            Self::TextContainsAllCaseInsensitive(list) => match value {
                Value::Text(t) => list
                    .iter()
                    .all(|s| t.to_lowercase().contains(&s.to_lowercase())),
                _ => false,
            },
            Self::TextContainsAny(list) => match value {
                Value::Text(t) => list.iter().any(|s| t.contains(s)),
                _ => false,
            },
            Self::TextContainsAnyCaseInsensitive(list) => match value {
                Value::Text(t) => list
                    .iter()
                    .any(|s| t.to_lowercase().contains(&s.to_lowercase())),
                _ => false,
            },
            Self::TextEndsWithAny(list) => match value {
                Value::Text(t) => list.iter().any(|s| t.ends_with(s)),
                _ => false,
            },
            Self::TextEndsWithAnyCaseInsensitive(list) => match value {
                Value::Text(t) => list
                    .iter()
                    .any(|s| t.to_lowercase().ends_with(&s.to_lowercase())),
                _ => false,
            },
            Self::TextStartsWithAny(list) => match value {
                Value::Text(t) => list.iter().any(|s| t.starts_with(s)),
                _ => false,
            },
            Self::TextStartsWithAnyCaseInsensitive(list) => match value {
                Value::Text(t) => list
                    .iter()
                    .any(|s| t.to_lowercase().starts_with(&s.to_lowercase())),
                _ => false,
            },
            Self::Xor(list) => {
                list.iter()
                    .fold(0, |acc, p| if p.check(value) { acc + 1 } else { acc })
                    == 1
            }
        }
    }

    /// Creates a new predicate for [`Predicate::Always`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::always();
    /// assert_eq!(p.check(&V::from(1)), true);
    /// ```
    #[inline]
    pub fn always() -> Self {
        Self::Always
    }

    /// Creates a new predicate for [`Predicate::Never`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::never();
    /// assert_eq!(p.check(&V::from(1)), false);
    /// ```
    #[inline]
    pub fn never() -> Self {
        Self::Never
    }

    /// Creates a new predicate for [`Predicate::Lambda`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::lambda(|v| v > &V::from(3));
    /// assert_eq!(p.check(&V::from(4)), true);
    /// assert_eq!(p.check(&V::from(1)), false);
    /// ```
    pub fn lambda<F: 'static + Fn(&Value) -> bool>(f: F) -> Self {
        Self::Lambda(Rc::new(f))
    }

    /// Creates a new predicate for [`Predicate::And`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::and(vec![
    ///     P::greater_than(1),
    ///     P::less_than(3),
    /// ]);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(1)), false);
    /// ```
    pub fn and<P: Into<Predicate>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::And(i.into_iter().map(|p| p.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::Not`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::not(P::greater_than(1));
    /// assert_eq!(p.check(&V::from(1)), true);
    /// assert_eq!(p.check(&V::from(2)), false);
    /// ```
    pub fn not<P: Into<Predicate>>(p: P) -> Self {
        Self::Not(Box::new(p.into()))
    }

    /// Creates a new predicate for [`Predicate::Or`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::or(vec![P::greater_than(1), P::equals(1)]);
    /// assert_eq!(p.check(&V::from(1)), true);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(0)), false);
    /// ```
    pub fn or<P: Into<Predicate>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::Or(i.into_iter().map(|p| p.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::Xor`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::xor(vec![P::greater_than(1), P::greater_than(2)]);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// assert_eq!(p.check(&V::from(1)), false);
    /// ```
    pub fn xor<P: Into<Predicate>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::Xor(i.into_iter().map(|p| p.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::Any`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::any(P::equals(3));
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), true);
    ///
    /// let p = P::any(P::equals(4));
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), false);
    /// ```
    pub fn any<P: Into<Predicate>>(p: P) -> Self {
        Self::Any(Box::new(p.into()))
    }

    /// Creates a new predicate for [`Predicate::Contains`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::contains(3);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), true);
    ///
    /// let p = P::contains(4);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), false);
    /// ```
    pub fn contains<T: Into<Value>>(value: T) -> Self {
        Self::Contains(value.into())
    }

    /// Creates a new predicate for [`Predicate::ContainsAll`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::contains_all(vec![1, 3]);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), true);
    ///
    /// let p = P::contains_all(vec![1, 4]);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), false);
    /// ```
    pub fn contains_all<T: Into<Value>, I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::ContainsAll(i.into_iter().map(|p| p.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::ContainsAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::contains_any(vec![1, 4]);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), true);
    ///
    /// let p = P::contains_any(vec![4, 5]);
    /// assert_eq!(p.check(&V::from(vec![1, 2, 3])), false);
    /// ```
    pub fn contains_any<T: Into<Value>, I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::ContainsAny(i.into_iter().map(|p| p.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::Equals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::equals(3);
    /// assert_eq!(p.check(&V::from(3)), true);
    /// assert_eq!(p.check(&V::from(2)), false);
    /// ```
    pub fn equals<T: Into<Value>>(value: T) -> Self {
        Self::Equals(value.into())
    }

    /// Creates a new predicate for [`Predicate::GreaterThan`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::greater_than(3);
    /// assert_eq!(p.check(&V::from(4)), true);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// ```
    pub fn greater_than<T: Into<Value>>(value: T) -> Self {
        Self::GreaterThan(value.into())
    }

    /// Creates a new predicate for [`Predicate::GreaterThanOrEquals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::greater_than_or_equals(3);
    /// assert_eq!(p.check(&V::from(4)), true);
    /// assert_eq!(p.check(&V::from(3)), true);
    /// assert_eq!(p.check(&V::from(2)), false);
    /// ```
    pub fn greater_than_or_equals<T: Into<Value>>(value: T) -> Self {
        Self::GreaterThanOrEquals(value.into())
    }

    /// Creates a new predicate for [`Predicate::InRange`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::in_range(3..=5);
    /// assert_eq!(p.check(&V::from(2)), false);
    /// assert_eq!(p.check(&V::from(3)), true);
    /// assert_eq!(p.check(&V::from(4)), true);
    /// assert_eq!(p.check(&V::from(5)), true);
    /// assert_eq!(p.check(&V::from(6)), false);
    /// ```
    pub fn in_range<T: Into<Value>>(range: RangeInclusive<T>) -> Self {
        let (start, end) = range.into_inner();
        Self::InRange(RangeInclusive::new(start.into(), end.into()))
    }

    /// Creates a new predicate for [`Predicate::InSet`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::in_set(vec![1, 2, 3]);
    /// assert_eq!(p.check(&V::from(0)), false);
    /// assert_eq!(p.check(&V::from(1)), true);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), true);
    /// assert_eq!(p.check(&V::from(4)), false);
    /// ```
    pub fn in_set<T: Into<Value>, I: IntoIterator<Item = T>>(set: I) -> Self {
        Self::InSet(set.into_iter().map(|v| v.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::LessThan`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::less_than(3);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// ```
    pub fn less_than<T: Into<Value>>(value: T) -> Self {
        Self::LessThan(value.into())
    }

    /// Creates a new predicate for [`Predicate::LessThanOrEquals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::less_than_or_equals(3);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), true);
    /// assert_eq!(p.check(&V::from(4)), false);
    /// ```
    pub fn less_than_or_equals<T: Into<Value>>(value: T) -> Self {
        Self::LessThanOrEquals(value.into())
    }

    /// Creates a new predicate for [`Predicate::NotEquals`]
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::not_equals(3);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// ```
    pub fn not_equals<T: Into<Value>>(value: T) -> Self {
        Self::NotEquals(value.into())
    }

    /// Creates a new predicate for [`Predicate::NotInRange`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::not_in_range(3..=5);
    /// assert_eq!(p.check(&V::from(2)), true);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// assert_eq!(p.check(&V::from(4)), false);
    /// assert_eq!(p.check(&V::from(5)), false);
    /// assert_eq!(p.check(&V::from(6)), true);
    /// ```
    pub fn not_in_range<T: Into<Value>>(range: RangeInclusive<T>) -> Self {
        let (start, end) = range.into_inner();
        Self::NotInRange(RangeInclusive::new(start.into(), end.into()))
    }

    /// Creates a new predicate for [`Predicate::NotInSet`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::not_in_set(vec![1, 2, 3]);
    /// assert_eq!(p.check(&V::from(0)), true);
    /// assert_eq!(p.check(&V::from(1)), false);
    /// assert_eq!(p.check(&V::from(2)), false);
    /// assert_eq!(p.check(&V::from(3)), false);
    /// assert_eq!(p.check(&V::from(4)), true);
    /// ```
    pub fn not_in_set<T: Into<Value>, I: IntoIterator<Item = T>>(set: I) -> Self {
        Self::NotInSet(set.into_iter().map(|v| v.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::HasKey`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    /// use std::collections::HashMap;
    ///
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    ///
    /// let p = P::has_key("a");
    /// assert_eq!(p.check(&V::from(map.clone())), true);
    ///
    /// let p = P::has_key("d");
    /// assert_eq!(p.check(&V::from(map)), false);
    /// ```
    pub fn has_key<K: Into<String>>(k: K) -> Self {
        Self::HasKey(k.into())
    }

    /// Creates a new typed predicate for [`Predicate::HasKeyWhereValue`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    /// use std::collections::HashMap;
    ///
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    ///
    /// let p = P::has_key_where_value("a", P::equals(1));
    /// assert_eq!(p.check(&V::from(map.clone())), true);
    ///
    /// let p = P::has_key_where_value("b", P::equals(1));
    /// assert_eq!(p.check(&V::from(map.clone())), false);
    ///
    /// let p = P::has_key_where_value("d", P::equals(1));
    /// assert_eq!(p.check(&V::from(map)), false);
    /// ```
    pub fn has_key_where_value<K: Into<String>, P: Into<Predicate>>(k: K, p: P) -> Self {
        Self::HasKeyWhereValue(k.into(), Box::new(p.into()))
    }

    /// Creates a new predicate for [`Predicate::IsNone`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::is_none();
    /// assert_eq!(p.check(&V::from(None::<u32>)), true);
    /// assert_eq!(p.check(&V::from(Some(3))), false);
    /// ```
    #[inline]
    pub fn is_none() -> Self {
        Self::IsNone
    }

    /// Creates a new predicate for [`Predicate::NotNoneAnd`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::not_none_and(P::equals(3));
    /// assert_eq!(p.check(&V::from(Some(3))), true);
    /// assert_eq!(p.check(&V::from(Some(2))), false);
    /// assert_eq!(p.check(&V::from(None::<u32>)), false);
    /// ```
    pub fn not_none_and<P: Into<Predicate>>(p: P) -> Self {
        Self::NotNoneAnd(Box::new(p.into()))
    }

    /// Creates a new predicate for [`Predicate::NoneOr`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::none_or(P::equals(3));
    /// assert_eq!(p.check(&V::from(Some(3))), true);
    /// assert_eq!(p.check(&V::from(None::<u32>)), true);
    /// assert_eq!(p.check(&V::from(Some(2))), false);
    /// ```
    pub fn none_or<P: Into<Predicate>>(p: P) -> Self {
        Self::NoneOr(Box::new(p.into()))
    }

    /// Creates a new predicate for [`Predicate::TextEndsWith`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_ends_with("text");
    ///
    /// assert_eq!(p.check(&V::from("some text")), true);
    /// assert_eq!(p.check(&V::from("text some")), false);
    /// ```
    pub fn text_ends_with<S: Into<String>>(s: S) -> Self {
        Self::TextEndsWith(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextEndsWithCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_ends_with_case_insensitive("text");
    ///
    /// assert_eq!(p.check(&V::from("SOME TEXT")), true);
    /// assert_eq!(p.check(&V::from("TEXT SOME")), false);
    /// ```
    pub fn text_ends_with_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::TextEndsWithCaseInsensitive(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextEqualsCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_equals_case_insensitive("text");
    ///
    /// assert_eq!(p.check(&V::from("TEXT")), true);
    /// assert_eq!(p.check(&V::from("OTHER")), false);
    /// ```
    pub fn text_equals_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::TextEqualsCaseInsensitive(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextInSetCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_in_set_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("TWO")), true);
    /// assert_eq!(p.check(&V::from("FOUR")), false);
    /// ```
    pub fn text_in_set_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::TextInSetCaseInsensitive(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextNotEqualsCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_not_equals_case_insensitive("text");
    ///
    /// assert_eq!(p.check(&V::from("OTHER")), true);
    /// assert_eq!(p.check(&V::from("TEXT")), false);
    /// ```
    pub fn text_not_equals_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::TextNotEqualsCaseInsensitive(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextStartsWith`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_starts_with("text");
    ///
    /// assert_eq!(p.check(&V::from("text some")), true);
    /// assert_eq!(p.check(&V::from("some text")), false);
    /// ```
    pub fn text_starts_with<S: Into<String>>(s: S) -> Self {
        Self::TextStartsWith(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextStartsWithCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_starts_with_case_insensitive("text");
    ///
    /// assert_eq!(p.check(&V::from("TEXT SOME")), true);
    /// assert_eq!(p.check(&V::from("SOME TEXT")), false);
    /// ```
    pub fn text_starts_with_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::TextStartsWithCaseInsensitive(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextContainedIn`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contained_in("text");
    ///
    /// assert_eq!(p.check(&V::from("ex")), true);
    /// assert_eq!(p.check(&V::from("tt")), false);
    /// ```
    pub fn text_contained_in<S: Into<String>>(s: S) -> Self {
        Self::TextContainedIn(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextContainedInCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contained_in_case_insensitive("text");
    ///
    /// assert_eq!(p.check(&V::from("EX")), true);
    /// assert_eq!(p.check(&V::from("TT")), false);
    /// ```
    pub fn text_contained_in_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::TextContainedInCaseInsensitive(s.into())
    }

    /// Creates a new predicate for [`Predicate::TextContainsAll`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contains_all(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("my one and two and three text")), true);
    /// assert_eq!(p.check(&V::from("my one and two text")), false);
    /// ```
    pub fn text_contains_all<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::TextContainsAll(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextContainsAllCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contains_all_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("MY ONE AND TWO AND THREE TEXT")), true);
    /// assert_eq!(p.check(&V::from("MY ONE AND TWO TEXT")), false);
    /// ```
    pub fn text_contains_all_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::TextContainsAllCaseInsensitive(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextContainsAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contains_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("my one text")), true);
    /// assert_eq!(p.check(&V::from("my four text")), false);
    /// ```
    pub fn text_contains_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::TextContainsAny(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextContainsAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_contains_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("MY ONE TEXT")), true);
    /// assert_eq!(p.check(&V::from("MY FOUR TEXT")), false);
    /// ```
    pub fn text_contains_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::TextContainsAnyCaseInsensitive(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextEndsWithAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_ends_with_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("my text one")), true);
    /// assert_eq!(p.check(&V::from("one my text")), false);
    /// ```
    pub fn text_ends_with_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::TextEndsWithAny(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextEndsWithAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_ends_with_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("MY TEXT ONE")), true);
    /// assert_eq!(p.check(&V::from("ONE MY TEXT")), false);
    /// ```
    pub fn text_ends_with_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::TextEndsWithAnyCaseInsensitive(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextStartsWithAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_starts_with_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("one my text")), true);
    /// assert_eq!(p.check(&V::from("my text one")), false);
    /// ```
    pub fn text_starts_with_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::TextStartsWithAny(i.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new predicate for [`Predicate::TextStartsWithAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate as P, Value as V};
    ///
    /// let p = P::text_starts_with_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(&V::from("ONE MY TEXT")), true);
    /// assert_eq!(p.check(&V::from("MY TEXT ONE")), false);
    /// ```
    pub fn text_starts_with_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::TextStartsWithAnyCaseInsensitive(i.into_iter().map(|s| s.into()).collect())
    }
}

impl<T: Into<Value>> PartialEq<TypedPredicate<T>> for Predicate {
    fn eq(&self, other: &TypedPredicate<T>) -> bool {
        self == other.as_untyped()
    }
}

impl<T: Into<Value>> From<TypedPredicate<T>> for Predicate {
    fn from(typed_predicate: TypedPredicate<T>) -> Self {
        typed_predicate.0
    }
}

impl std::ops::BitXor for Predicate {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Xor`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(123)) ^ Predicate::Equals(Value::from(124)),
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the left-hand side is already a [`Predicate::Xor`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///     ]) ^ Predicate::Equals(Value::from(124)),
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the right-hand side is already a [`Predicate::Xor`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(122)) ^ Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If both sides are already a [`Predicate::Xor`], the returned
    /// predicate will be a merge:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///     ]) ^ Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::Xor(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    fn bitxor(self, rhs: Self) -> Self {
        let inner = match (self, rhs) {
            (Self::Xor(mut list1), Self::Xor(mut list2)) => {
                list1.append(&mut list2);
                list1
            }
            (x, Self::Xor(mut list)) => {
                list.insert(0, x);
                list
            }
            (Self::Xor(mut list), x) => {
                list.push(x);
                list
            }
            (x1, x2) => vec![x1, x2],
        };
        Self::Xor(inner)
    }
}

impl std::ops::BitAnd for Predicate {
    type Output = Self;

    /// Shorthand to produce [`Predicate::And`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(123)) & Predicate::Equals(Value::from(124)),
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the left-hand side is already a [`Predicate::And`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///     ]) & Predicate::Equals(Value::from(124)),
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the right-hand side is already a [`Predicate::And`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(122)) & Predicate::And(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If both sides are already a [`Predicate::And`], the returned
    /// predicate will be a merge:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///     ]) & Predicate::And(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::And(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    fn bitand(self, rhs: Self) -> Self {
        let inner = match (self, rhs) {
            (Self::And(mut list1), Self::And(mut list2)) => {
                list1.append(&mut list2);
                list1
            }
            (x, Self::And(mut list)) => {
                list.insert(0, x);
                list
            }
            (Self::And(mut list), x) => {
                list.push(x);
                list
            }
            (x1, x2) => vec![x1, x2],
        };
        Self::And(inner)
    }
}

impl std::ops::BitOr for Predicate {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Or`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(123)) | Predicate::Equals(Value::from(124)),
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the left-hand side is already a [`Predicate::Or`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///     ]) | Predicate::Equals(Value::from(124)),
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If the right-hand side is already a [`Predicate::Or`], the returned
    /// predicate will be an updated instance:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Equals(Value::from(122)) | Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    ///
    /// If both sides are already a [`Predicate::Or`], the returned
    /// predicate will be a merge:
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///     ]) | Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    ///     Predicate::Or(vec![
    ///         Predicate::Equals(Value::from(121)),
    ///         Predicate::Equals(Value::from(122)),
    ///         Predicate::Equals(Value::from(123)),
    ///         Predicate::Equals(Value::from(124)),
    ///     ]),
    /// );
    /// ```
    fn bitor(self, rhs: Self) -> Self {
        let inner = match (self, rhs) {
            (Self::Or(mut list1), Self::Or(mut list2)) => {
                list1.append(&mut list2);
                list1
            }
            (x, Self::Or(mut list)) => {
                list.insert(0, x);
                list
            }
            (Self::Or(mut list), x) => {
                list.push(x);
                list
            }
            (x1, x2) => vec![x1, x2],
        };
        Self::Or(inner)
    }
}

impl std::ops::Not for Predicate {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Not`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{Predicate, Value};
    ///
    /// assert_eq!(
    ///     !Predicate::Equals(Value::from(123)),
    ///     Predicate::Not(Box::new(Predicate::Equals(Value::from(123)))),
    /// );
    /// ```
    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

/// Represents a typed [`Predicate`], ensuring that only valid conditions are
/// used for a given type
#[derive(Clone, Debug, PartialEq)]
pub struct TypedPredicate<T: Into<Value>>(Predicate, PhantomData<T>);

impl<T: Into<Value>> PartialEq<Predicate> for TypedPredicate<T> {
    fn eq(&self, other: &Predicate) -> bool {
        self.as_untyped() == other
    }
}

impl<T: Into<Value>> From<Predicate> for TypedPredicate<T> {
    fn from(predicate: Predicate) -> Self {
        Self(predicate, PhantomData)
    }
}

impl<T: Into<Value>> std::ops::BitXor for TypedPredicate<T> {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Xor`]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.0 ^ rhs.0)
    }
}

impl<T: Into<Value>> std::ops::BitAnd for TypedPredicate<T> {
    type Output = Self;

    /// Shorthand to produce [`Predicate::And`]
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.0 & rhs.0)
    }
}

impl<T: Into<Value>> std::ops::BitOr for TypedPredicate<T> {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Or`]
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.0 | rhs.0)
    }
}

impl<T: Into<Value>> std::ops::Not for TypedPredicate<T> {
    type Output = Self;

    /// Shorthand to produce [`Predicate::Not`]
    fn not(self) -> Self::Output {
        Self::new(!self.0)
    }
}

impl<T: Into<Value> + TryFrom<Value>> TypedPredicate<T> {
    /// Creates a new typed predicate for [`Predicate::Lambda`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::lambda(|x| x > 3);
    /// assert_eq!(p.check(4), true);
    /// assert_eq!(p.check(1), false);
    /// ```
    pub fn lambda<F: 'static + Fn(T) -> bool>(f: F) -> Self {
        Self::new(Predicate::Lambda(Rc::new(move |v| {
            match T::try_from(v.clone()) {
                Ok(x) => f(x),
                Err(_) => false,
            }
        })))
    }
}

impl<T: Into<Value>> TypedPredicate<T> {
    /// Creates a new typed predicate from an untyped predicate
    pub fn new(predicate: Predicate) -> Self {
        Self::from(predicate)
    }

    /// Returns a reference to the untyped [`Predicate`] wrapped by this
    /// typed instance
    #[inline]
    pub fn as_untyped(&self) -> &Predicate {
        &self.0
    }

    /// Checks if the typed predicate is satisfied by the given value
    ///
    /// NOTE: This consumes the value instead of the untyped version that
    ///       merely references the value
    pub fn check(&self, value: T) -> bool {
        self.0.check(&value.into())
    }

    /// Creates a new typed predicate for [`Predicate::Always`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::always();
    /// assert_eq!(p.check(1), true);
    /// ```
    pub fn always() -> Self {
        Self::new(Predicate::always())
    }

    /// Creates a new typed predicate for [`Predicate::Never`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::never();
    /// assert_eq!(p.check(1), false);
    /// ```
    pub fn never() -> Self {
        Self::new(Predicate::never())
    }

    /// Creates a new typed predicate for [`Predicate::And`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::and(vec![
    ///     P::greater_than(1),
    ///     P::less_than(3),
    /// ]);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(1), false);
    /// ```
    pub fn and<P: Into<TypedPredicate<T>>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::new(Predicate::and::<TypedPredicate<T>, Vec<TypedPredicate<T>>>(
            i.into_iter().map(|p| p.into()).collect(),
        ))
    }

    /// Creates a new typed predicate for [`Predicate::Not`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::not(P::greater_than(1));
    /// assert_eq!(p.check(1), true);
    /// assert_eq!(p.check(2), false);
    /// ```
    pub fn not<P: Into<TypedPredicate<T>>>(p: P) -> Self {
        Self::new(Predicate::not(p.into()))
    }

    /// Creates a new typed predicate for [`Predicate::Or`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::or(vec![P::greater_than(1), P::equals(1)]);
    /// assert_eq!(p.check(1), true);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(0), false);
    /// ```
    pub fn or<P: Into<TypedPredicate<T>>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::new(Predicate::or::<TypedPredicate<T>, Vec<TypedPredicate<T>>>(
            i.into_iter().map(|p| p.into()).collect(),
        ))
    }

    /// Creates a new typed predicate for [`Predicate::Xor`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::xor(vec![P::greater_than(1), P::greater_than(2)]);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), false);
    /// assert_eq!(p.check(1), false);
    /// ```
    pub fn xor<P: Into<TypedPredicate<T>>, I: IntoIterator<Item = P>>(i: I) -> Self {
        Self::new(Predicate::xor::<TypedPredicate<T>, Vec<TypedPredicate<T>>>(
            i.into_iter().map(|p| p.into()).collect(),
        ))
    }
}

/// Implementation for collections with a singular type. For
/// [`std::collections::HashMap`] and similar types, use the
/// [`MapTypedPredicate`] instead.
impl<T: Into<Value>, C: IntoIterator<Item = T> + Into<Value>> TypedPredicate<C> {
    /// Creates a new typed predicate for [`Predicate::Any`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::any(P::equals(3));
    /// assert_eq!(p.check(vec![1, 2, 3]), true);
    ///
    /// let p = P::any(P::equals(4));
    /// assert_eq!(p.check(vec![1, 2, 3]), false);
    /// ```
    pub fn any<P: Into<TypedPredicate<T>>>(p: P) -> Self {
        Self::new(Predicate::any(p.into()))
    }

    /// Creates a new typed predicate for [`Predicate::Contains`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::contains(3);
    /// assert_eq!(p.check(vec![1, 2, 3]), true);
    ///
    /// let p = P::contains(4);
    /// assert_eq!(p.check(vec![1, 2, 3]), false);
    /// ```
    pub fn contains(value: T) -> Self {
        Self::new(Predicate::contains(value))
    }

    /// Creates a new typed predicate for [`Predicate::ContainsAll`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::contains_all(vec![1, 3]);
    /// assert_eq!(p.check(vec![1, 2, 3]), true);
    ///
    /// let p = P::contains_all(vec![1, 4]);
    /// assert_eq!(p.check(vec![1, 2, 3]), false);
    /// ```
    pub fn contains_all<I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::new(Predicate::contains_all(i))
    }

    /// Creates a new typed predicate for [`Predicate::ContainsAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::contains_any(vec![1, 4]);
    /// assert_eq!(p.check(vec![1, 2, 3]), true);
    ///
    /// let p = P::contains_any(vec![4, 5]);
    /// assert_eq!(p.check(vec![1, 2, 3]), false);
    /// ```
    pub fn contains_any<I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::new(Predicate::contains_any(i))
    }
}

impl<T: Into<Value>> TypedPredicate<T> {
    /// Creates a new typed predicate for [`Predicate::Equals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::equals(3);
    /// assert_eq!(p.check(3), true);
    /// assert_eq!(p.check(2), false);
    /// ```
    pub fn equals(value: T) -> Self {
        Self::new(Predicate::equals(value))
    }

    /// Creates a new typed predicate for [`Predicate::GreaterThan`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::greater_than(3);
    /// assert_eq!(p.check(4), true);
    /// assert_eq!(p.check(3), false);
    /// ```
    pub fn greater_than(value: T) -> Self {
        Self::new(Predicate::greater_than(value))
    }

    /// Creates a new typed predicate for [`Predicate::GreaterThanOrEquals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::greater_than_or_equals(3);
    /// assert_eq!(p.check(4), true);
    /// assert_eq!(p.check(3), true);
    /// assert_eq!(p.check(2), false);
    /// ```
    pub fn greater_than_or_equals(value: T) -> Self {
        Self::new(Predicate::greater_than_or_equals(value))
    }

    /// Creates a new typed predicate for [`Predicate::InRange`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::in_range(3..=5);
    /// assert_eq!(p.check(2), false);
    /// assert_eq!(p.check(3), true);
    /// assert_eq!(p.check(4), true);
    /// assert_eq!(p.check(5), true);
    /// assert_eq!(p.check(6), false);
    /// ```
    pub fn in_range(range: RangeInclusive<T>) -> Self {
        Self::new(Predicate::in_range(range))
    }

    /// Creates a new typed predicate for [`Predicate::InSet`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::in_set(vec![1, 2, 3]);
    /// assert_eq!(p.check(0), false);
    /// assert_eq!(p.check(1), true);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), true);
    /// assert_eq!(p.check(4), false);
    /// ```
    pub fn in_set<I: IntoIterator<Item = T>>(set: I) -> Self {
        Self::new(Predicate::in_set(set))
    }

    /// Creates a new typed predicate for [`Predicate::LessThan`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::less_than(3);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), false);
    /// ```
    pub fn less_than(value: T) -> Self {
        Self::new(Predicate::less_than(value))
    }

    /// Creates a new typed predicate for [`Predicate::LessThanOrEquals`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::less_than_or_equals(3);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), true);
    /// assert_eq!(p.check(4), false);
    /// ```
    pub fn less_than_or_equals(value: T) -> Self {
        Self::new(Predicate::less_than_or_equals(value))
    }

    /// Creates a new typed predicate for [`Predicate::NotEquals`]
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::not_equals(3);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), false);
    /// ```
    pub fn not_equals(value: T) -> Self {
        Self::new(Predicate::not_equals(value))
    }

    /// Creates a new typed predicate for [`Predicate::NotInRange`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::not_in_range(3..=5);
    /// assert_eq!(p.check(2), true);
    /// assert_eq!(p.check(3), false);
    /// assert_eq!(p.check(4), false);
    /// assert_eq!(p.check(5), false);
    /// assert_eq!(p.check(6), true);
    /// ```
    pub fn not_in_range(range: RangeInclusive<T>) -> Self {
        Self::new(Predicate::not_in_range(range))
    }

    /// Creates a new typed predicate for [`Predicate::NotInSet`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::not_in_set(vec![1, 2, 3]);
    /// assert_eq!(p.check(0), true);
    /// assert_eq!(p.check(1), false);
    /// assert_eq!(p.check(2), false);
    /// assert_eq!(p.check(3), false);
    /// assert_eq!(p.check(4), true);
    /// ```
    pub fn not_in_set<I: IntoIterator<Item = T>>(set: I) -> Self {
        Self::new(Predicate::not_in_set(set))
    }
}

impl<T: Into<Value>> TypedPredicate<HashMap<String, T>> {
    /// Creates a new typed predicate for [`Predicate::HasKey`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    /// use std::collections::HashMap;
    ///
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    ///
    /// let p = P::has_key("a");
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = P::has_key("d");
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn has_key<K: Into<String>>(k: K) -> Self {
        Self::new(Predicate::has_key(k))
    }

    /// Creates a new typed predicate for [`Predicate::HasKeyWhereValue`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    /// use std::collections::HashMap;
    ///
    /// let map: HashMap<String, u32> = vec![
    ///     (String::from("a"), 1),
    ///     (String::from("b"), 2),
    ///     (String::from("c"), 3),
    /// ].into_iter().collect();
    ///
    /// let p = P::has_key_where_value("a", P::equals(1));
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = P::has_key_where_value("b", P::equals(1));
    /// assert_eq!(p.check(map.clone()), false);
    ///
    /// let p = P::has_key_where_value("d", P::equals(1));
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn has_key_where_value<K: Into<String>, P: Into<TypedPredicate<T>>>(k: K, p: P) -> Self {
        Self::new(Predicate::has_key_where_value(k.into(), p.into()))
    }
}

impl<T: Into<Value>> TypedPredicate<Option<T>> {
    /// Creates a new typed predicate for [`Predicate::IsNone`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::is_none();
    ///
    /// let v = None::<u32>;
    /// assert_eq!(p.check(v), true);
    ///
    /// let v = Some(3);
    /// assert_eq!(p.check(v), false);
    /// ```
    pub fn is_none() -> Self {
        Self::new(Predicate::IsNone)
    }

    /// Creates a new typed predicate for [`Predicate::NotNoneAnd`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::not_none_and(P::equals(3));
    ///
    /// let v = Some(3);
    /// assert_eq!(p.check(v), true);
    ///
    /// let v = Some(2);
    /// assert_eq!(p.check(v), false);
    ///
    /// let v = None::<u32>;
    /// assert_eq!(p.check(v), false);
    /// ```
    pub fn not_none_and<P: Into<TypedPredicate<T>>>(p: P) -> Self {
        Self::new(Predicate::not_none_and(p.into()))
    }

    /// Creates a new typed predicate for [`Predicate::NoneOr`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::none_or(P::equals(3));
    ///
    /// let v = Some(3);
    /// assert_eq!(p.check(v), true);
    ///
    /// let v = None::<u32>;
    /// assert_eq!(p.check(v), true);
    ///
    /// let v = Some(2);
    /// assert_eq!(p.check(v), false);
    /// ```
    pub fn none_or<P: Into<TypedPredicate<T>>>(p: P) -> Self {
        Self::new(Predicate::none_or(p.into()))
    }
}

impl TypedPredicate<String> {
    /// Creates a new typed predicate for [`Predicate::TextEndsWith`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_ends_with("text");
    ///
    /// assert_eq!(p.check(String::from("some text")), true);
    /// assert_eq!(p.check(String::from("text some")), false);
    /// ```
    pub fn text_ends_with<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_ends_with(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextEndsWithCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_ends_with_case_insensitive("text");
    ///
    /// assert_eq!(p.check(String::from("SOME TEXT")), true);
    /// assert_eq!(p.check(String::from("TEXT SOME")), false);
    /// ```
    pub fn text_ends_with_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_ends_with_case_insensitive(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextEqualsCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_equals_case_insensitive("text");
    ///
    /// assert_eq!(p.check(String::from("TEXT")), true);
    /// assert_eq!(p.check(String::from("OTHER")), false);
    /// ```
    pub fn text_equals_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_equals_case_insensitive(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextInSetCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_in_set_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("TWO")), true);
    /// assert_eq!(p.check(String::from("FOUR")), false);
    /// ```
    pub fn text_in_set_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::new(Predicate::text_in_set_case_insensitive(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextNotEqualsCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_not_equals_case_insensitive("text");
    ///
    /// assert_eq!(p.check(String::from("OTHER")), true);
    /// assert_eq!(p.check(String::from("TEXT")), false);
    /// ```
    pub fn text_not_equals_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_not_equals_case_insensitive(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextStartsWith`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_starts_with("text");
    ///
    /// assert_eq!(p.check(String::from("text some")), true);
    /// assert_eq!(p.check(String::from("some text")), false);
    /// ```
    pub fn text_starts_with<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_starts_with(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextStartsWithCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_starts_with_case_insensitive("text");
    ///
    /// assert_eq!(p.check(String::from("TEXT SOME")), true);
    /// assert_eq!(p.check(String::from("SOME TEXT")), false);
    /// ```
    pub fn text_starts_with_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_starts_with_case_insensitive(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainedIn`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contained_in("text");
    ///
    /// assert_eq!(p.check(String::from("ex")), true);
    /// assert_eq!(p.check(String::from("tt")), false);
    /// ```
    pub fn text_contained_in<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_contained_in(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainedInCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contained_in_case_insensitive("text");
    ///
    /// assert_eq!(p.check(String::from("EX")), true);
    /// assert_eq!(p.check(String::from("TT")), false);
    /// ```
    pub fn text_contained_in_case_insensitive<S: Into<String>>(s: S) -> Self {
        Self::new(Predicate::text_contained_in_case_insensitive(s))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainsAll`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contains_all(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("my one and two and three text")), true);
    /// assert_eq!(p.check(String::from("my one and two text")), false);
    /// ```
    pub fn text_contains_all<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::new(Predicate::text_contains_all(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainsAllCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contains_all_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("MY ONE AND TWO AND THREE TEXT")), true);
    /// assert_eq!(p.check(String::from("MY ONE AND TWO TEXT")), false);
    /// ```
    pub fn text_contains_all_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::new(Predicate::text_contains_all_case_insensitive(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainsAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contains_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("my one text")), true);
    /// assert_eq!(p.check(String::from("my four text")), false);
    /// ```
    pub fn text_contains_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::new(Predicate::text_contains_any(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextContainsAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_contains_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("MY ONE TEXT")), true);
    /// assert_eq!(p.check(String::from("MY FOUR TEXT")), false);
    /// ```
    pub fn text_contains_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::new(Predicate::text_contains_any_case_insensitive(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextEndsWithAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_ends_with_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("my text one")), true);
    /// assert_eq!(p.check(String::from("one my text")), false);
    /// ```
    pub fn text_ends_with_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::new(Predicate::text_ends_with_any(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextEndsWithAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_ends_with_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("MY TEXT ONE")), true);
    /// assert_eq!(p.check(String::from("ONE MY TEXT")), false);
    /// ```
    pub fn text_ends_with_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::new(Predicate::text_ends_with_any_case_insensitive(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextStartsWithAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_starts_with_any(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("one my text")), true);
    /// assert_eq!(p.check(String::from("my text one")), false);
    /// ```
    pub fn text_starts_with_any<S: Into<String>, I: IntoIterator<Item = S>>(i: I) -> Self {
        Self::new(Predicate::text_starts_with_any(i))
    }

    /// Creates a new typed predicate for [`Predicate::TextStartsWithAnyCaseInsensitive`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::TypedPredicate as P;
    ///
    /// let p = P::text_starts_with_any_case_insensitive(vec!["one", "two", "three"]);
    ///
    /// assert_eq!(p.check(String::from("ONE MY TEXT")), true);
    /// assert_eq!(p.check(String::from("MY TEXT ONE")), false);
    /// ```
    pub fn text_starts_with_any_case_insensitive<S: Into<String>, I: IntoIterator<Item = S>>(
        i: I,
    ) -> Self {
        Self::new(Predicate::text_starts_with_any_case_insensitive(i))
    }
}

/// Represents a typed [`Predicate`] specifically for maps such as
/// [`std::collections::HashMap`], ensuring that only valid conditions are
/// used for a given type.
///
/// This is required due to limitations in Rust's blanket impl functionality,
/// which will be resolved once specialization is available.
#[derive(Clone, Debug, PartialEq)]
pub struct MapTypedPredicate<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>>(
    Predicate,
    PhantomData<T>,
    PhantomData<C>,
);

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> PartialEq<Predicate>
    for MapTypedPredicate<T, C>
{
    fn eq(&self, other: &Predicate) -> bool {
        self.as_untyped() == other
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> From<Predicate>
    for MapTypedPredicate<T, C>
{
    fn from(predicate: Predicate) -> Self {
        Self(predicate, PhantomData, PhantomData)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>>
    From<MapTypedPredicate<T, C>> for Predicate
{
    fn from(map_typed_predicate: MapTypedPredicate<T, C>) -> Self {
        map_typed_predicate.0
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> From<TypedPredicate<C>>
    for MapTypedPredicate<T, C>
{
    fn from(typed_predicate: TypedPredicate<C>) -> Self {
        Self(typed_predicate.into(), PhantomData, PhantomData)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>>
    From<MapTypedPredicate<T, C>> for TypedPredicate<C>
{
    fn from(map_typed_predicate: MapTypedPredicate<T, C>) -> Self {
        Self(map_typed_predicate.into(), PhantomData)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> std::ops::BitXor
    for MapTypedPredicate<T, C>
{
    type Output = Self;

    /// Shorthand to produce [`Predicate::Xor`]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.0 ^ rhs.0)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> std::ops::BitAnd
    for MapTypedPredicate<T, C>
{
    type Output = Self;

    /// Shorthand to produce [`Predicate::And`]
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.0 & rhs.0)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> std::ops::BitOr
    for MapTypedPredicate<T, C>
{
    type Output = Self;

    /// Shorthand to produce [`Predicate::Or`]
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.0 | rhs.0)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> std::ops::Not
    for MapTypedPredicate<T, C>
{
    type Output = Self;

    /// Shorthand to produce [`Predicate::Not`]
    fn not(self) -> Self::Output {
        Self::new(!self.0)
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> MapTypedPredicate<T, C> {
    /// Creates a new map typed predicate from an untyped predicate
    pub fn new(predicate: Predicate) -> Self {
        Self::from(predicate)
    }

    /// Returns a reference to the untyped [`Predicate`] wrapped by this
    /// typed instance
    #[inline]
    pub fn as_untyped(&self) -> &Predicate {
        &self.0
    }

    /// Checks if the typed predicate is satisfied by the given value
    ///
    /// NOTE: This consumes the value instead of the untyped version that
    ///       merely references the value
    pub fn check(&self, value: C) -> bool {
        self.0.check(&value.into())
    }
}

impl<T: Into<Value>, C: IntoIterator<Item = (String, T)> + Into<Value>> MapTypedPredicate<T, C> {
    /// Creates a new typed predicate for [`Predicate::Any`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{TypedPredicate as P, MapTypedPredicate as MP};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(String::from("a"), 1);
    /// map.insert(String::from("b"), 2);
    /// map.insert(String::from("c"), 3);
    ///
    /// let p = MP::any(P::equals(3));
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = MP::any(P::equals(4));
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn any<P: Into<TypedPredicate<T>>>(p: P) -> Self {
        Self::new(Predicate::any(p.into()))
    }

    /// Creates a new typed predicate for [`Predicate::Contains`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{TypedPredicate as P, MapTypedPredicate as MP};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(String::from("a"), 1);
    /// map.insert(String::from("b"), 2);
    /// map.insert(String::from("c"), 3);
    ///
    /// let p = MP::contains(3);
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = MP::contains(4);
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn contains(value: T) -> Self {
        Self::new(Predicate::contains(value))
    }

    /// Creates a new typed predicate for [`Predicate::ContainsAll`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{TypedPredicate as P, MapTypedPredicate as MP};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(String::from("a"), 1);
    /// map.insert(String::from("b"), 2);
    /// map.insert(String::from("c"), 3);
    ///
    /// let p = MP::contains_all(vec![1, 3]);
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = MP::contains_all(vec![1, 4]);
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn contains_all<I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::new(Predicate::contains_all(i))
    }

    /// Creates a new typed predicate for [`Predicate::ContainsAny`]
    ///
    /// ### Examples
    ///
    /// ```
    /// use entity::{TypedPredicate as P, MapTypedPredicate as MP};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(String::from("a"), 1);
    /// map.insert(String::from("b"), 2);
    /// map.insert(String::from("c"), 3);
    ///
    /// let p = MP::contains_any(vec![1, 4]);
    /// assert_eq!(p.check(map.clone()), true);
    ///
    /// let p = MP::contains_any(vec![4, 5]);
    /// assert_eq!(p.check(map), false);
    /// ```
    pub fn contains_any<I: IntoIterator<Item = T>>(i: I) -> Self {
        Self::new(Predicate::contains_any(i))
    }
}
