use derive_more::From;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};
use strum::{Display, EnumDiscriminants, EnumString, ParseError};

/// Represents either a primitive or complex value
#[derive(Clone, Debug, PartialEq, Eq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Optional(Option<Box<Value>>),
    Primitive(PrimitiveValue),
    Set(HashSet<PrimitiveValue>),
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

            // Compare text
            (Self::Text(a), Self::Text(b)) => a.partial_cmp(b),

            // All other types do nothing
            _ => None,
        }
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

impl From<Option<Value>> for Value {
    /// Converts optional value into a value by moving value into heap
    fn from(maybe_value: Option<Value>) -> Self {
        Self::from(maybe_value.map(Box::from))
    }
}

impl<'a> From<&'a str> for Value {
    /// Converts a str slice into a value by allocating a new string
    fn from(s: &'a str) -> Self {
        Self::from(s.to_string())
    }
}

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
    /// use entity::{ValueType as VT, PrimitiveValueType as PVT};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(VT::from_str("u32").unwrap(), VT::Primitive(PVT::U32));
    /// assert_eq!(VT::from_str("list:u32").unwrap(), VT::List(Box::from(VT::Primitive(PVT::U32))));
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
            Value::Set(x) => Self::Set(
                x.iter()
                    .next()
                    .map(PrimitiveValueType::from)
                    .unwrap_or_default(),
            ),
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
#[derive(Copy, Clone, Debug, From, EnumDiscriminants)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(Display, EnumString))]
#[strum_discriminants(name(PrimitiveValueType), strum(serialize_all = "snake_case"))]
#[cfg_attr(
    feature = "serde",
    strum_discriminants(derive(serde::Serialize, serde::Deserialize))
)]
pub enum PrimitiveValue {
    Bool(bool),
    Char(char),
    F32(f32),
    F64(f64),
    I128(i128),
    I16(i16),
    I32(i32),
    I64(i64),
    I8(i8),
    Isize(isize),
    U128(u128),
    U16(u16),
    U32(u32),
    U64(u64),
    U8(u8),
    Unit,
    Usize(usize),
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

/// Value is considered equal, ignoring the fact that NaN != NaN for floats
impl Eq for PrimitiveValue {}

impl PartialEq for PrimitiveValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => a.to_string() == b.to_string(),
            (Self::F64(a), Self::F64(b)) => a.to_string() == b.to_string(),
            (Self::I128(a), Self::I128(b)) => a == b,
            (Self::I16(a), Self::I16(b)) => a == b,
            (Self::I32(a), Self::I32(b)) => a == b,
            (Self::I64(a), Self::I64(b)) => a == b,
            (Self::I8(a), Self::I8(b)) => a == b,
            (Self::Isize(a), Self::Isize(b)) => a == b,
            (Self::U128(a), Self::U128(b)) => a == b,
            (Self::U16(a), Self::U16(b)) => a == b,
            (Self::U32(a), Self::U32(b)) => a == b,
            (Self::U64(a), Self::U64(b)) => a == b,
            (Self::U8(a), Self::U8(b)) => a == b,
            (Self::Unit, Self::Unit) => true,
            (Self::Usize(a), Self::Usize(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for PrimitiveValue {
    /// Hashes the value, converting f32/f64 into a string before doing so
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Bool(x) => x.hash(state),
            Self::Char(x) => x.hash(state),
            Self::F32(x) => x.to_string().hash(state),
            Self::F64(x) => x.to_string().hash(state),
            Self::I128(x) => x.hash(state),
            Self::I16(x) => x.hash(state),
            Self::I32(x) => x.hash(state),
            Self::I64(x) => x.hash(state),
            Self::I8(x) => x.hash(state),
            Self::Isize(x) => x.hash(state),
            Self::U128(x) => x.hash(state),
            Self::U16(x) => x.hash(state),
            Self::U32(x) => x.hash(state),
            Self::U64(x) => x.hash(state),
            Self::U8(x) => x.hash(state),
            Self::Unit => Self::Unit.hash(state),
            Self::Usize(x) => x.hash(state),
        }
    }
}

impl PartialOrd for PrimitiveValue {
    /// Compares same variants for ordering, otherwise returns none
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Char(a), Self::Char(b)) => a.partial_cmp(b),
            (Self::F32(a), Self::F32(b)) => a.partial_cmp(b),
            (Self::F64(a), Self::F64(b)) => a.partial_cmp(b),
            (Self::I128(a), Self::I128(b)) => a.partial_cmp(b),
            (Self::I16(a), Self::I16(b)) => a.partial_cmp(b),
            (Self::I32(a), Self::I32(b)) => a.partial_cmp(b),
            (Self::I64(a), Self::I64(b)) => a.partial_cmp(b),
            (Self::I8(a), Self::I8(b)) => a.partial_cmp(b),
            (Self::Isize(a), Self::Isize(b)) => a.partial_cmp(b),
            (Self::U128(a), Self::U128(b)) => a.partial_cmp(b),
            (Self::U16(a), Self::U16(b)) => a.partial_cmp(b),
            (Self::U32(a), Self::U32(b)) => a.partial_cmp(b),
            (Self::U64(a), Self::U64(b)) => a.partial_cmp(b),
            (Self::U8(a), Self::U8(b)) => a.partial_cmp(b),
            (Self::Usize(a), Self::Usize(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
