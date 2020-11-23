use crate::ent::value::Value;

/// Represents a condition on an ent's field
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldCondition {
    /// Query condition that succeeds if the ent's field value is not a
    /// collection and succeeds with the given condition applied to it
    Value(ValueCondition),

    /// Query condition that succeeds if the ent's field value is a collection
    /// that succeeds with the given condition applied to its keys (if map)
    CollectionKey(CollectionCondition),

    /// Query condition that succeeds if the ent's field value is a collection
    /// that succeeds with the given condition applied to its values
    CollectionValue(CollectionCondition),
}

impl FieldCondition {
    /// Checks if the given value passes the condition. Additionally, if a
    /// value is not appropriate for this condition, false will be returned.
    pub fn check(&self, value: &Value) -> bool {
        match self {
            Self::Value(c) => c.check(value),
            Self::CollectionKey(c) => c.check_keys(value),
            Self::CollectionValue(c) => c.check(value),
        }
    }
}

impl From<ValueCondition> for FieldCondition {
    /// Converts a value condition into a field condition over a value
    fn from(c: ValueCondition) -> Self {
        Self::Value(c)
    }
}

impl From<CollectionCondition> for FieldCondition {
    /// Converts a collection condition into a field condition over a
    /// collection's values
    fn from(c: CollectionCondition) -> Self {
        Self::CollectionValue(c)
    }
}

/// Represents a condition on an ent's field's value that does not represent
/// a collection
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValueCondition {
    /// Query condition that succeeds if the ent's field is less than the
    /// specified field value
    LessThan(Value),

    /// Query condition that succeeds if the ent's field is equal to the
    /// specified field value
    EqualTo(Value),

    /// Query condition that succeeds if the ent's field is greater than the
    /// specified field value
    GreaterThan(Value),
}

impl ValueCondition {
    /// Shorthand to produce a value condition that checks if the ent's value
    /// is equal to the provided value
    #[inline]
    pub fn equal_to<V: Into<Value>>(v: V) -> Self {
        Self::EqualTo(v.into())
    }

    /// Shorthand to produce a value condition that checks if the ent's value
    /// is greater than the provided value
    #[inline]
    pub fn greater_than<V: Into<Value>>(v: V) -> Self {
        Self::GreaterThan(v.into())
    }

    /// Shorthand to produce a value condition that checks if the ent's value
    /// is less than the provided value
    #[inline]
    pub fn less_than<V: Into<Value>>(v: V) -> Self {
        Self::LessThan(v.into())
    }

    /// Returns a reference to the value represented by the condition
    #[inline]
    pub fn value(&self) -> &Value {
        match self {
            Self::LessThan(v) => v,
            Self::EqualTo(v) => v,
            Self::GreaterThan(v) => v,
        }
    }

    /// Checks if the given value passes the condition. Additionally, if a
    /// value is not appropriate for this condition, false will be returned.
    #[inline]
    pub fn check(&self, value: &Value) -> bool {
        match self {
            Self::EqualTo(v) => value == v,
            Self::GreaterThan(v) => value > v,
            Self::LessThan(v) => value < v,
        }
    }
}

/// Represents a condition on an ent's field's value that represents a collection
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CollectionCondition {
    /// For all values within the collection, check if at least one passes the condition
    Any(Box<FieldCondition>),

    /// For all values within the collection, check if exactly N passes the condition
    Exactly(Box<FieldCondition>, usize),

    /// For all values within the collection, check all pass the condition
    All(Box<FieldCondition>),

    /// Collection length (usize) must pass the value condition
    Len(ValueCondition),
}

impl CollectionCondition {
    /// Shorthand to produce a collection condition that checks if any value
    /// in a collection passes the field condition
    #[inline]
    pub fn any<C: Into<FieldCondition>>(c: C) -> Self {
        Self::Any(Box::from(c.into()))
    }

    /// Shorthand to produce a collection condition that checks if exactly
    /// n values in a collection passes the field condition
    #[inline]
    pub fn exactly<C: Into<FieldCondition>>(c: C, n: usize) -> Self {
        Self::Exactly(Box::from(c.into()), n)
    }

    /// Shorthand to produce a collection condition that checks if all values
    /// in a collection passes the field condition
    #[inline]
    pub fn all<C: Into<FieldCondition>>(c: C) -> Self {
        Self::All(Box::from(c.into()))
    }

    /// Shorthand to produce a collection condition that checks the length of
    /// the collection passes the given value condition
    #[inline]
    pub fn len<C: Into<ValueCondition>>(c: C) -> Self {
        Self::Len(c.into())
    }

    /// Checks if the given value passes the condition. Additionally, if a
    /// value is not appropriate for this condition, false will be returned.
    ///
    /// In this case, the value is assumed to be some type that can be iterated,
    /// which includes maps, lists, and optional values (whose inner value) is
    /// iterated exactly once.
    ///
    /// ## Examples
    ///
    /// For a list and a condition of any, each value is checked to see if it
    /// passes the condition. The check stops as soon as the first pass is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::List(vec![Value::from(1), Value::from(2), Value::from(3)]);
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(2));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(4));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a list and a condition of exactly, each value is checked to see if it
    /// passes the condition. This will check all values, figure out how many
    /// passed, and return whether or not that equals n.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::List(vec![Value::from(1), Value::from(2), Value::from(3)]);
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than(1), 2);
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than(2), 2);
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a list and a condition of all, each value is checked to see if it
    /// passes the condition. This check stops as soon as the first fail is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::List(vec![Value::from(1), Value::from(2), Value::from(3)]);
    ///
    /// let c = CollectionCondition::all(ValueCondition::greater_than(0));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::greater_than(1));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a list and condition of len, the length of the list is
    /// checked to see if it passes the condition.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::List(vec![Value::from(1), Value::from(2), Value::from(3)]);
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(4));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(1));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a map and a condition of any, each value is checked to see if it
    /// passes the condition. The check stops as soon as the first pass is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(2));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(4));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a map and a condition of exactly, each value is checked to see if it
    /// passes the condition. This will check all values, figure out how many
    /// passed, and return whether or not that equals n.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than(1), 2);
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than(2), 2);
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a map and a condition of all, each value is checked to see if it
    /// passes the condition. This check stops as soon as the first fail is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::all(ValueCondition::greater_than(0));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::greater_than(1));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For a map and condition of len, the length of the map is
    /// checked to see if it passes the condition.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(4));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(1));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For an optional, its inner value is checked by treating the optional
    /// as a list with one element. This means that any and all conditions
    /// behave the same and the exactly condition will only succeed if n = 1
    /// for some or n = 0 for none.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    /// let v = Value::Optional(Some(Box::from(Value::from(3))));
    ///
    /// // Any and all behave the same here
    /// let c = CollectionCondition::any(ValueCondition::equal_to(3));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(4));
    /// assert!(!c.check(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::equal_to(3));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::equal_to(4));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    /// let v = Value::Optional(Some(Box::from(Value::from(3))));
    ///
    /// // Exactly must both pass and have n = 1 to succeed
    /// let c = CollectionCondition::exactly(ValueCondition::equal_to(3), 1);
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::equal_to(3), 2);
    /// assert!(!c.check(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::equal_to(4), 1);
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Optional(None);
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to(3));
    /// assert!(!c.check(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::equal_to(3), 1);
    /// assert!(!c.check(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::equal_to(3));
    /// assert!(!c.check(&v));
    ///
    /// let c = CollectionCondition::len(ValueCondition::equal_to(0));
    /// assert!(c.check(&v));
    ///
    /// let c = CollectionCondition::len(ValueCondition::equal_to(1));
    /// assert!(!c.check(&v));
    /// ```
    ///
    /// For primitives and text, the condition will automatically fail.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value, PrimitiveValue};
    ///
    /// let v = Value::Text(String::from("a"));
    /// let c = CollectionCondition::any(ValueCondition::less_than("d"));
    /// assert!(!c.check(&v));
    /// let c = CollectionCondition::exactly(ValueCondition::less_than("d"), 1);
    /// assert!(!c.check(&v));
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check(&v));
    ///
    /// let v = Value::Primitive(PrimitiveValue::Char('a'));
    /// let c = CollectionCondition::any(ValueCondition::less_than("d"));
    /// assert!(!c.check(&v));
    /// let c = CollectionCondition::exactly(ValueCondition::less_than("d"), 1);
    /// assert!(!c.check(&v));
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check(&v));
    /// ```
    pub fn check(&self, value: &Value) -> bool {
        fn any<'a>(c: &FieldCondition, mut it: impl Iterator<Item = &'a Value>) -> bool {
            it.any(|v| c.check(v))
        }

        fn exactly<'a>(c: &FieldCondition, n: usize, it: impl Iterator<Item = &'a Value>) -> bool {
            it.filter(|v| c.check(v)).count() == n
        }

        fn all<'a>(c: &FieldCondition, it: impl Iterator<Item = &'a Value>) -> bool {
            let (passing, total) = it.fold((0, 0), |(mut passing, mut total), v| {
                if c.check(v) {
                    passing += 1;
                }
                total += 1;
                (passing, total)
            });

            total > 0 && passing == total
        }

        match (self, value) {
            (CollectionCondition::Any(c), Value::List(list)) => any(c, list.iter()),
            (CollectionCondition::Any(c), Value::Map(map)) => any(c, map.values()),
            (CollectionCondition::Any(c), Value::Optional(o)) => {
                any(c, o.iter().map(AsRef::as_ref))
            }

            (CollectionCondition::Exactly(c, n), Value::List(list)) => exactly(c, *n, list.iter()),
            (CollectionCondition::Exactly(c, n), Value::Map(map)) => exactly(c, *n, map.values()),
            (CollectionCondition::Exactly(c, n), Value::Optional(o)) => {
                exactly(c, *n, o.iter().map(AsRef::as_ref))
            }

            (CollectionCondition::All(c), Value::List(list)) => all(c, list.iter()),
            (CollectionCondition::All(c), Value::Map(map)) => all(c, map.values()),
            (CollectionCondition::All(c), Value::Optional(o)) => {
                all(c, o.iter().map(AsRef::as_ref))
            }

            (CollectionCondition::Len(cond), Value::List(list)) => {
                cond.check(&Value::from(list.len()))
            }
            (CollectionCondition::Len(cond), Value::Map(map)) => {
                cond.check(&Value::from(map.len()))
            }
            (CollectionCondition::Len(cond), Value::Optional(o)) => {
                cond.check(&Value::from(if o.is_some() { 1usize } else { 0usize }))
            }

            _ => false,
        }
    }

    /// Checks if the given value passes the condition. Additionally, if a
    /// value is not appropriate for this condition, false will be returned.
    ///
    /// In this case, the value is assumed to be a map that has keys that will
    /// be validated. If it is not a map, false will always be returned.
    ///
    /// ## Examples
    ///
    /// For a map and a condition of any, each key is checked to see if it
    /// passes the condition. The check stops as soon as the first pass is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to("c"));
    /// assert!(c.check_keys(&v));
    ///
    /// let c = CollectionCondition::any(ValueCondition::equal_to("d"));
    /// assert!(!c.check_keys(&v));
    /// ```
    ///
    /// For a map and a condition of exactly, each key is checked to see if it
    /// passes the condition. This will check all values, figure out how many
    /// passed, and return whether or not that equals n.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than("a"), 2);
    /// assert!(c.check_keys(&v));
    ///
    /// let c = CollectionCondition::exactly(ValueCondition::greater_than("b"), 2);
    /// assert!(!c.check_keys(&v));
    /// ```
    ///
    /// For a map and a condition of all, each key is checked to see if it
    /// passes the condition. This check stops as soon as the first fail is
    /// found or all list elements have been checked.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(c.check_keys(&v));
    ///
    /// let c = CollectionCondition::all(ValueCondition::less_than("c"));
    /// assert!(!c.check_keys(&v));
    /// ```
    ///
    /// For a map and condition of len, the length of the map's keys is
    /// checked to see if it passes the condition.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value};
    ///
    /// let v = Value::Map(vec![
    ///     (String::from("a"), Value::from(1)),
    ///     (String::from("b"), Value::from(2)),
    ///     (String::from("c"), Value::from(3)),
    /// ].into_iter().collect());
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(4));
    /// assert!(c.check_keys(&v));
    ///
    /// let c = CollectionCondition::len(ValueCondition::less_than(1));
    /// assert!(!c.check_keys(&v));
    /// ```
    ///
    /// Any other value that is not a map will fail the condition, regardless
    /// of any, exactly, or all.
    ///
    /// ```
    /// use entity::{CollectionCondition, ValueCondition, Value, PrimitiveValue};
    ///
    /// // List has no keys
    /// let v = Value::List(vec![Value::from("a"), Value::from("b"), Value::from("c")]);
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check_keys(&v));
    ///
    /// // Optional has no keys
    /// let v = Value::Optional(Some(Box::from(Value::from("a"))));
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check_keys(&v));
    ///
    /// // Text has no keys
    /// let v = Value::Text(String::from("a"));
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check_keys(&v));
    ///
    /// // Primitive has no keys
    /// let v = Value::Primitive(PrimitiveValue::Char('a'));
    /// let c = CollectionCondition::all(ValueCondition::less_than("d"));
    /// assert!(!c.check_keys(&v));
    /// ```
    pub fn check_keys(&self, value: &Value) -> bool {
        match (self, value) {
            (CollectionCondition::Any(cond), Value::Map(map)) => {
                map.keys().any(|k| cond.check(&Value::Text(k.to_string())))
            }
            (CollectionCondition::Exactly(cond, n), Value::Map(map)) => {
                map.keys()
                    .filter(|k| cond.check(&Value::Text(k.to_string())))
                    .count()
                    == *n
            }
            (CollectionCondition::All(cond), Value::Map(map)) => {
                !map.is_empty() && map.keys().all(|k| cond.check(&Value::Text(k.to_string())))
            }
            (CollectionCondition::Len(cond), Value::Map(map)) => {
                cond.check(&Value::from(map.len()))
            }
            _ => false,
        }
    }
}
