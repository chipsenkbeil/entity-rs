mod number;
pub use number::{Number, NumberSign, NumberType};

use derive_more::From;
use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::{Hash, Hasher},
};
use strum::ParseError;

/// Represents either a primitive or complex value
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Optional(Option<Box<Value>>),
    Primitive(PrimitiveValue),
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
    pub fn to_primitive(&self) -> Option<PrimitiveValue> {
        match self {
            Self::Primitive(x) => Some(*x),
            _ => None,
        }
    }

    /// Converts into underlying primitive type if representing one
    #[inline]
    pub fn to_primitive_type(&self) -> Option<PrimitiveValueType> {
        self.to_primitive().map(PrimitiveValueType::from)
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

            // Compare primitives based on primitive value ordering
            (Self::Primitive(a), Self::Primitive(b)) => a.partial_cmp(b),

            // Compare text-to-text, text-to-char, and char-to-text
            (Self::Text(a), Self::Text(b)) => a.partial_cmp(b),
            (Self::Text(a), Self::Primitive(PrimitiveValue::Char(b))) => {
                a.partial_cmp(&b.to_string())
            }
            (Self::Primitive(PrimitiveValue::Char(a)), Self::Text(b)) => {
                a.to_string().partial_cmp(b)
            }

            // All other types do nothing
            _ => None,
        }
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Converts a vec of some value into a value list
    fn from(list: Vec<T>) -> Self {
        Self::List(list.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<Value>> From<HashMap<String, T>> for Value {
    /// Converts a hashmap of string keys and some value into a value map
    fn from(map: HashMap<String, T>) -> Self {
        Self::Map(map.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    /// Converts an option of some value into an optional value
    fn from(maybe: Option<T>) -> Self {
        Self::Optional(maybe.map(|x| Box::from(x.into())))
    }
}

impl From<PrimitiveValue> for Value {
    /// Converts a primitive value into a value without any allocation
    fn from(v: PrimitiveValue) -> Self {
        Self::Primitive(v)
    }
}

impl From<String> for Value {
    /// Converts a string into a text value without any allocation
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl<'a> From<&'a str> for Value {
    /// Converts a str slice into a value by allocating a new string
    fn from(s: &'a str) -> Self {
        Self::from(s.to_string())
    }
}

macro_rules! impl_from_primitive {
    ($type:ty) => {
        impl From<$type> for Value {
            fn from(v: $type) -> Self {
                Self::from(PrimitiveValue::from(v))
            }
        }
    };
}

impl_from_primitive!(bool);
impl_from_primitive!(char);
impl_from_primitive!(f32);
impl_from_primitive!(f64);
impl_from_primitive!(i128);
impl_from_primitive!(i16);
impl_from_primitive!(i32);
impl_from_primitive!(i64);
impl_from_primitive!(i8);
impl_from_primitive!(isize);
impl_from_primitive!(u128);
impl_from_primitive!(u16);
impl_from_primitive!(u32);
impl_from_primitive!(u64);
impl_from_primitive!(u8);
impl_from_primitive!(usize);

/// Represents value types (primitive or complex). Assumes that complex
/// types will contain the same inner type and does not vary
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValueType {
    List(Box<ValueType>),
    Map(Box<ValueType>),
    Optional(Box<ValueType>),
    Primitive(PrimitiveValueType),
    Set(PrimitiveValueType),
    Text,
}

impl ValueType {
    pub fn is_primitive_type(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    pub fn to_primitive_type(&self) -> Option<PrimitiveValueType> {
        match self {
            Self::Primitive(x) => Some(*x),
            _ => None,
        }
    }
}

impl Default for ValueType {
    /// Returns default value type of primitive unit
    fn default() -> Self {
        Self::Primitive(Default::default())
    }
}

impl std::str::FromStr for ValueType {
    type Err = ParseError;

    /// Parses a string delimited by colons into a nested value type
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{ValueType as VT, PrimitiveValueType as PVT, NumberType as NT};
    /// use strum::ParseError;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(VT::from_str("char").unwrap(), VT::Primitive(PVT::Char));
    /// assert_eq!(VT::from_str("u32").unwrap(), VT::Primitive(PVT::Number(NT::U32)));
    /// assert_eq!(VT::from_str("number:u32").unwrap(), VT::Primitive(PVT::Number(NT::U32)));
    /// assert_eq!(VT::from_str("primitive:number:u32").unwrap(), VT::Primitive(PVT::Number(NT::U32)));
    /// assert_eq!(VT::from_str("list:u32").unwrap(), VT::List(Box::from(VT::Primitive(PVT::Number(NT::U32)))));
    /// assert_eq!(VT::from_str("list:number:u32").unwrap(), VT::List(Box::from(VT::Primitive(PVT::Number(NT::U32)))));
    /// assert_eq!(VT::from_str("list:primitive:number:u32").unwrap(), VT::List(Box::from(VT::Primitive(PVT::Number(NT::U32)))));
    /// assert_eq!(VT::from_str("unknown").unwrap_err(), ParseError::VariantNotFound);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn opt_to_err(maybe_type: Option<ValueType>) -> Result<ValueType, ParseError> {
            match maybe_type {
                Some(t) => Ok(t),
                None => Err(ParseError::VariantNotFound),
            }
        }

        fn from_tokens<'a>(
            mut it: impl Iterator<Item = &'a str>,
        ) -> Result<Option<ValueType>, ParseError> {
            match it.next() {
                // Special case where we cannot feed this directly into the
                // primitive value type as it is the following type that is
                // used instead, so we take the next value instead and use it
                Some("number") => from_tokens(it),
                Some(token) => {
                    let maybe_inner = from_tokens(it)?;
                    match token {
                        "list" => Ok(Some(ValueType::List(Box::from(opt_to_err(maybe_inner)?)))),
                        "map" => Ok(Some(ValueType::Map(Box::from(opt_to_err(maybe_inner)?)))),
                        "optional" => Ok(Some(ValueType::Optional(Box::from(opt_to_err(
                            maybe_inner,
                        )?)))),
                        "primitive" => Ok(Some(ValueType::Primitive(
                            opt_to_err(maybe_inner)?
                                .to_primitive_type()
                                .ok_or(ParseError::VariantNotFound)?,
                        ))),
                        "set" => Ok(Some(ValueType::Set(
                            opt_to_err(maybe_inner)?
                                .to_primitive_type()
                                .ok_or(ParseError::VariantNotFound)?,
                        ))),
                        "text" => Ok(Some(ValueType::Text)),
                        x => Ok(Some(ValueType::Primitive(PrimitiveValueType::from_str(x)?))),
                    }
                }
                None => Ok(None),
            }
        }

        match from_tokens(s.split(':')) {
            Ok(Some(value_type)) => Ok(value_type),
            Ok(None) => Err(ParseError::VariantNotFound),
            Err(x) => Err(x),
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(t) => write!(f, "list:{}", t),
            Self::Map(t) => write!(f, "map:{}", t),
            Self::Optional(t) => write!(f, "optional:{}", t),
            Self::Primitive(t) => write!(f, "{}", t),
            Self::Set(t) => write!(f, "set:{}", t),
            Self::Text => write!(f, "text"),
        }
    }
}

impl<'a> From<&'a Value> for ValueType {
    /// Produces the type of the referenced value by recursively iterating
    /// through complex types, assuming that the first value in types like
    /// list represent the entire set, defaulting to a primitive unit if
    /// a complex value does not have any items
    fn from(v: &'a Value) -> Self {
        match v {
            Value::List(x) => Self::List(Box::from(
                x.iter().next().map(ValueType::from).unwrap_or_default(),
            )),
            Value::Map(x) => Self::Map(Box::from(
                x.values().next().map(ValueType::from).unwrap_or_default(),
            )),
            Value::Optional(x) => Self::Optional(Box::from(
                x.as_ref()
                    .map(Box::as_ref)
                    .map(ValueType::from)
                    .unwrap_or_default(),
            )),
            Value::Primitive(x) => Self::Primitive(PrimitiveValueType::from(x)),
            Value::Text(_) => Self::Text,
        }
    }
}

impl From<PrimitiveValueType> for ValueType {
    /// Converts primitive value type to a value type
    fn from(t: PrimitiveValueType) -> Self {
        Self::Primitive(t)
    }
}

impl Default for PrimitiveValueType {
    /// Returns default primitive value type of unit
    fn default() -> Self {
        Self::Unit
    }
}

/// Represents a primitive value
#[derive(Copy, Clone, Debug, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveValue {
    Bool(bool),
    Char(char),
    Number(Number),
    Unit,
}

impl PrimitiveValue {
    /// Returns true if this value is of the specified type
    #[inline]
    pub fn is_type(&self, r#type: PrimitiveValueType) -> bool {
        self.to_type() == r#type
    }

    /// Returns the type of this value
    #[inline]
    pub fn to_type(&self) -> PrimitiveValueType {
        PrimitiveValueType::from(self)
    }

    /// Returns true if this value and the other value are of the same type
    #[inline]
    pub fn has_same_type(&self, other: &PrimitiveValue) -> bool {
        self.to_type() == other.to_type()
    }
}
impl Hash for PrimitiveValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Bool(x) => x.hash(state),
            Self::Char(x) => x.hash(state),
            Self::Number(x) => x.hash(state),
            Self::Unit => Self::Unit.hash(state),
        }
    }
}

/// Value is considered equal, ignoring the fact that NaN != NaN for floats
impl Eq for PrimitiveValue {}

impl PartialEq for PrimitiveValue {
    /// Compares two primitive values of same type for equality, otherwise
    /// returns false
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Unit, Self::Unit) => true,
            _ => false,
        }
    }
}

impl PartialOrd for PrimitiveValue {
    /// Compares same variants of same type for ordering, otherwise returns none
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Char(a), Self::Char(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Unit, Self::Unit) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

macro_rules! impl_to_number {
    ($type:ty) => {
        impl From<$type> for PrimitiveValue {
            fn from(v: $type) -> Self {
                Self::Number(Number::from(v))
            }
        }
    };
}

impl_to_number!(f32);
impl_to_number!(f64);
impl_to_number!(i128);
impl_to_number!(i16);
impl_to_number!(i32);
impl_to_number!(i64);
impl_to_number!(i8);
impl_to_number!(isize);
impl_to_number!(u128);
impl_to_number!(u16);
impl_to_number!(u32);
impl_to_number!(u64);
impl_to_number!(u8);
impl_to_number!(usize);

/// Represents primitive value types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveValueType {
    Bool,
    Char,
    Number(NumberType),
    Unit,
}

impl PrimitiveValueType {
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn is_char(&self) -> bool {
        matches!(self, Self::Char)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn to_number_type(&self) -> Option<NumberType> {
        match self {
            Self::Number(x) => Some(*x),
            _ => None,
        }
    }
}

impl std::fmt::Display for PrimitiveValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Char => write!(f, "char"),
            Self::Number(t) => write!(f, "number:{}", t),
            Self::Unit => write!(f, "unit"),
        }
    }
}

impl From<PrimitiveValue> for PrimitiveValueType {
    fn from(v: PrimitiveValue) -> Self {
        Self::from(&v)
    }
}

impl<'a> From<&'a PrimitiveValue> for PrimitiveValueType {
    fn from(v: &'a PrimitiveValue) -> Self {
        match v {
            PrimitiveValue::Bool(_) => Self::Bool,
            PrimitiveValue::Char(_) => Self::Char,
            PrimitiveValue::Number(x) => Self::Number(x.to_type()),
            PrimitiveValue::Unit => Self::Unit,
        }
    }
}

impl std::str::FromStr for PrimitiveValueType {
    type Err = ParseError;

    /// Parses a primitive value type
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{PrimitiveValueType as PVT, NumberType as NT};
    /// use strum::ParseError;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(PVT::from_str("bool").unwrap(), PVT::Bool);
    /// assert_eq!(PVT::from_str("char").unwrap(), PVT::Char);
    /// assert_eq!(PVT::from_str("u32").unwrap(), PVT::Number(NT::U32));
    /// assert_eq!(PVT::from_str("number:u32").unwrap(), PVT::Number(NT::U32));
    /// assert_eq!(PVT::from_str("unit").unwrap(), PVT::Unit);
    /// assert_eq!(PVT::from_str("unknown").unwrap_err(), ParseError::VariantNotFound);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s_it = s.split(':');
        let primary = s_it.next();
        let secondary = s_it.next();
        let has_more = s_it.next().is_some();

        // If has too many values, we exit
        if has_more {
            return Err(ParseError::VariantNotFound);
        }

        match (primary, secondary) {
            (Some("bool"), None) => Ok(Self::Bool),
            (Some("char"), None) => Ok(Self::Char),
            (Some("number"), Some(x)) => Ok(Self::Number(NumberType::from_str(x)?)),
            (Some("unit"), None) => Ok(Self::Unit),
            (Some(x), None) => Ok(Self::Number(NumberType::from_str(x)?)),
            _ => Err(ParseError::VariantNotFound),
        }
    }
}
