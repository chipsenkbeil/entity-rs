mod number;
mod primitive;
mod r#type;

pub use number::{Number, NumberLike, NumberSign, NumberType};
pub use primitive::{Primitive, PrimitiveLike, PrimitiveType};
pub use r#type::ValueType;

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    convert::TryFrom,
    ffi::{OsStr, OsString},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

/// Represents either a primitive or complex value
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Optional(Option<Box<Value>>),
    Primitive(Primitive),
    Text(String),
}

impl Value {
    /// Returns true if this value is of the specified type
    #[inline]
    pub fn is_type(&self, r#type: ValueType) -> bool {
        self.to_type() == r#type
    }

    /// Returns the type of this value
    #[inline]
    pub fn to_type(&self) -> ValueType {
        ValueType::from(self)
    }

    /// Returns true if this value and the other value are of the same type
    #[inline]
    pub fn has_same_type(&self, other: &Value) -> bool {
        self.to_type() == other.to_type()
    }

    /// Returns true if not representing a primitive value
    #[inline]
    pub fn is_complex(&self) -> bool {
        !self.is_primitive()
    }

    /// Returns true if representing a primitive value
    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// Converts into underlying primitive value if representing one
    #[inline]
    pub fn to_primitive(&self) -> Option<Primitive> {
        match self {
            Self::Primitive(x) => Some(*x),
            _ => None,
        }
    }

    /// Converts into underlying primitive type if representing one
    #[inline]
    pub fn to_primitive_type(&self) -> Option<PrimitiveType> {
        self.to_primitive().map(PrimitiveType::from)
    }

    /// Attempts to convert the value to an underlying option type,
    /// succeeding if Value is the Optional variant and the inner
    /// value can be converted to the specified type.
    ///
    /// This is only needed due to a blanket impl in the standard library
    /// blocking the ability to implement `TryFrom<Value> for Option<T>`,
    /// which will be available some day once specialization is implemented:
    ///
    /// https://github.com/rust-lang/rust/issues/31844
    pub fn try_into_option<T: TryFrom<Value, Error = &'static str>>(
        self,
    ) -> Result<Option<T>, &'static str> {
        match self {
            Self::Optional(Some(boxed_value)) => {
                let t = T::try_from(boxed_value.as_ref().clone())?;
                Ok(Some(t))
            }
            Self::Optional(None) => Ok(None),
            _ => Err("Only Optional can be converted to Option<T>"),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::List(x) => x.hash(state),
            Self::Map(x) => {
                let mut keys = x.keys().collect::<Vec<&String>>();
                keys.sort_unstable();
                keys.hash(state);

                // TODO: Is there a better way to approach hashing when a value
                //       might not support ordering? Should we filter out all
                //       values that are not comparable? If so, we would need
                //       to provide some method on value, primitive, and number
                //       that can tell us if it is comparable
                let mut values = x.values().collect::<Vec<&Value>>();
                values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
                values.hash(state);
            }
            Self::Optional(x) => x.hash(state),
            Self::Primitive(x) => x.hash(state),
            Self::Text(x) => x.hash(state),
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    /// Compares two values of same type for equality, otherwise
    /// returns false
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            (Self::Optional(a), Self::Optional(b)) => a == b,
            (Self::Optional(Some(a)), b) => a.as_ref() == b,
            (a, Self::Optional(Some(b))) => a == b.as_ref(),
            (Self::Primitive(a), Self::Primitive(b)) => a == b,
            (Self::Text(a), Self::Text(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    /// Compares same variants for ordering, otherwise returns none
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // Compare lists lexographically
            (Self::List(a), Self::List(b)) => a.partial_cmp(b),

            // Compare elements inside options if both are available
            (Self::Optional(a), Self::Optional(b)) => match (a, b) {
                (Some(a), Some(b)) => a.partial_cmp(b),
                _ => None,
            },

            // Compare value inside option against any other type
            (Self::Optional(Some(a)), b) => a.as_ref().partial_cmp(b),
            (a, Self::Optional(Some(b))) => a.partial_cmp(b.as_ref()),

            // Compare primitives based on primitive value ordering
            (Self::Primitive(a), Self::Primitive(b)) => a.partial_cmp(b),

            // Compare text-to-text, text-to-char, and char-to-text
            (Self::Text(a), Self::Text(b)) => a.partial_cmp(b),
            (Self::Text(a), Self::Primitive(Primitive::Char(b))) => a.partial_cmp(&b.to_string()),
            (Self::Primitive(Primitive::Char(a)), Self::Text(b)) => a.to_string().partial_cmp(b),

            // All other types do nothing
            _ => None,
        }
    }
}

/// Represents some data that can be converted to and from a [`Value`]
pub trait ValueLike: Sized {
    /// Consumes this data, converting it into an abstract [`Value`]
    fn into_value(self) -> Value;

    /// Attempts to convert an abstract [`Value`] into this data, returning
    /// the owned value back if unable to convert
    fn try_from_value(value: Value) -> Result<Self, Value>;
}

impl ValueLike for Value {
    fn into_value(self) -> Value {
        self
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        Ok(value)
    }
}

impl<T: PrimitiveLike> ValueLike for T {
    fn into_value(self) -> Value {
        Value::Primitive(self.into_primitive())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Primitive(x) => T::try_from_primitive(x).map_err(Value::Primitive),
            x => Err(x),
        }
    }
}

impl<T: ValueLike> ValueLike for Option<T> {
    fn into_value(self) -> Value {
        Value::Optional(self.map(|x| Box::new(x.into_value())))
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Optional(Some(x)) => Ok(Some(
                T::try_from_value(*x).map_err(|x| Value::Optional(Some(Box::new(x))))?,
            )),
            Value::Optional(None) => Ok(None),
            x => Err(x),
        }
    }
}

impl ValueLike for PathBuf {
    fn into_value(self) -> Value {
        Value::Text(self.to_string_lossy().to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => Ok(PathBuf::from(x)),
            x => Err(x),
        }
    }
}

impl ValueLike for OsString {
    fn into_value(self) -> Value {
        Value::Text(self.to_string_lossy().to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => Ok(OsString::from(x)),
            x => Err(x),
        }
    }
}

impl ValueLike for String {
    fn into_value(self) -> Value {
        Value::Text(self)
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => Ok(x),
            x => Err(x),
        }
    }
}

impl<'a> From<&'a Path> for Value {
    fn from(p: &'a Path) -> Self {
        p.to_path_buf().into_value()
    }
}

impl<'a> From<&'a OsStr> for Value {
    fn from(s: &'a OsStr) -> Self {
        s.to_os_string().into_value()
    }
}

impl<'a> From<&'a str> for Value {
    fn from(s: &'a str) -> Self {
        s.to_string().into_value()
    }
}

macro_rules! impl_list {
    ($outer:ident $($type:tt)*) => {
        impl<T: ValueLike $(+ $type)*> ValueLike for $outer<T> {
            fn into_value(self) -> Value {
                Value::List(self.into_iter().map(ValueLike::into_value).collect())
            }

            fn try_from_value(value: Value) -> Result<Self, Value> {
                match value {
                    Value::List(x) => {
                        let mut tmp = Vec::new();
                        let mut has_failure = false;

                        for v in x {
                            let result = T::try_from_value(v);
                            if result.is_err() {
                                has_failure = true;
                            }
                            tmp.push(result);
                        }

                        // Roll back to the original value list if there is
                        // any error
                        if has_failure {
                            Err(Value::List(
                                tmp.into_iter()
                                    .map(|v| match v {
                                        Ok(x) => x.into_value(),
                                        Err(x) => x,
                                    })
                                    .collect(),
                            ))
                        } else {
                            Ok(tmp.into_iter().map(|v| v.unwrap()).collect())
                        }
                    }
                    x => Err(x),
                }
            }
        }
    };
}

impl_list!(Vec);
impl_list!(VecDeque);
impl_list!(LinkedList);
impl_list!(BinaryHeap Ord);
impl_list!(HashSet Hash Eq);
impl_list!(BTreeSet Ord);

macro_rules! impl_map {
    ($outer:ident) => {
        impl<T: ValueLike> ValueLike for $outer<String, T> {
            fn into_value(self) -> Value {
                Value::Map(self.into_iter().map(|(k, v)| (k, v.into_value())).collect())
            }

            fn try_from_value(value: Value) -> Result<Self, Value> {
                match value {
                    Value::Map(x) => {
                        let mut tmp = Vec::new();
                        let mut has_failure = false;

                        for (k, v) in x {
                            let result = match T::try_from_value(v) {
                                Ok(v) => Ok((k, v)),
                                Err(v) => Err((k, v)),
                            };
                            if result.is_err() {
                                has_failure = true;
                            }
                            tmp.push(result);
                        }

                        // Roll back to the original value list if there is
                        // any error
                        if has_failure {
                            Err(Value::Map(
                                tmp.into_iter()
                                    .map(|x| match x {
                                        Ok((k, v)) => (k, v.into_value()),
                                        Err((k, v)) => (k, v),
                                    })
                                    .collect(),
                            ))
                        } else {
                            Ok(tmp.into_iter().map(|v| v.unwrap()).collect())
                        }
                    }
                    x => Err(x),
                }
            }
        }
    };
}

impl_map!(HashMap);
impl_map!(BTreeMap);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_can_convert_to_value() {
        assert_eq!(true.into_value(), Value::Primitive(Primitive::Bool(true)));
    }

    #[test]
    fn value_can_convert_to_bool() {
        assert_eq!(
            bool::try_from_value(Value::Primitive(Primitive::Bool(true))),
            Ok(true),
        );

        assert_eq!(
            bool::try_from_value(Value::Primitive(Primitive::Char('c'))),
            Err(Value::Primitive(Primitive::Char('c'))),
        );
    }

    #[test]
    fn char_can_convert_to_value() {
        assert_eq!('c'.into_value(), Value::Primitive(Primitive::Char('c')));
    }

    #[test]
    fn value_can_convert_to_char() {
        todo!();
    }

    #[test]
    fn f32_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_f32() {
        todo!();
    }

    #[test]
    fn f64_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_f64() {
        todo!();
    }

    #[test]
    fn isize_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_isize() {
        todo!();
    }

    #[test]
    fn i8_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_i8() {
        todo!();
    }

    #[test]
    fn i16_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_i16() {
        todo!();
    }

    #[test]
    fn i32_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_i32() {
        todo!();
    }

    #[test]
    fn i64_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_i64() {
        todo!();
    }

    #[test]
    fn i128_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_i128() {
        todo!();
    }

    #[test]
    fn usize_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_usize() {
        todo!();
    }

    #[test]
    fn u8_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_u8() {
        todo!();
    }

    #[test]
    fn u16_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_u16() {
        todo!();
    }

    #[test]
    fn u32_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_u32() {
        todo!();
    }

    #[test]
    fn u64_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_u64() {
        todo!();
    }

    #[test]
    fn u128_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_u128() {
        todo!();
    }

    #[test]
    fn option_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_option() {
        todo!();
    }

    #[test]
    fn string_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_string() {
        todo!();
    }

    #[test]
    fn pathbuf_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_pathbuf() {
        todo!();
    }

    #[test]
    fn osstring_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_osstring() {
        todo!();
    }

    #[test]
    fn vec_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_vec() {
        todo!();
    }

    #[test]
    fn vecdeque_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_vecdeque() {
        todo!();
    }

    #[test]
    fn linkedlist_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_linkedlist() {
        todo!();
    }

    #[test]
    fn binaryheap_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_binaryheap() {
        todo!();
    }

    #[test]
    fn hashset_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_hashset() {
        todo!();
    }

    #[test]
    fn btreeset_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_btreeset() {
        todo!();
    }

    #[test]
    fn hashmap_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_hashmap() {
        todo!();
    }

    #[test]
    fn btreemap_can_convert_to_value() {
        todo!();
    }

    #[test]
    fn value_can_convert_to_btreemap() {
        todo!();
    }
}
