use super::{Number, NumberLike, NumberType};

use std::{
    cmp::Ordering,
    convert::TryFrom,
    hash::{Hash, Hasher},
};
use strum::ParseError;

/// Represents a primitive value
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveValue {
    Bool(bool),
    Char(char),
    Number(Number),
    Unit,
}

impl Default for PrimitiveValueType {
    /// Returns default primitive value type of unit
    fn default() -> Self {
        Self::Unit
    }
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

/// Represents some data that can be converted to and from a [`PrimitiveValue`]
pub trait PrimitiveValueLike: Sized {
    /// Consumes this data, converting it into an abstract [`PrimitiveValue`]
    fn into_primitive_value(self) -> PrimitiveValue;

    /// Attempts to convert an abstract [`PrimitiveValue`] into this data, returning
    /// the owned value back if unable to convert
    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue>;
}

impl PrimitiveValueLike for PrimitiveValue {
    fn into_primitive_value(self) -> PrimitiveValue {
        self
    }

    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue> {
        Ok(primitive_value)
    }
}

impl PrimitiveValueLike for bool {
    fn into_primitive_value(self) -> PrimitiveValue {
        PrimitiveValue::Bool(self)
    }

    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue> {
        match primitive_value {
            PrimitiveValue::Bool(x) => Ok(x),
            x => Err(x),
        }
    }
}

impl PrimitiveValueLike for char {
    fn into_primitive_value(self) -> PrimitiveValue {
        PrimitiveValue::Char(self)
    }

    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue> {
        match primitive_value {
            PrimitiveValue::Char(x) => Ok(x),
            x => Err(x),
        }
    }
}

impl<T: NumberLike> PrimitiveValueLike for T {
    fn into_primitive_value(self) -> PrimitiveValue {
        PrimitiveValue::Number(self.into_number())
    }

    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue> {
        match primitive_value {
            PrimitiveValue::Number(x) => T::try_from_number(x).map_err(PrimitiveValue::Number),
            x => Err(x),
        }
    }
}

macro_rules! impl_conv {
    ($($type:ty)+) => {$(
        impl From<$type> for PrimitiveValue {
            fn from(x: $type) -> Self {
                <$type as PrimitiveValueLike>::into_primitive_value(x)
            }
        }

        impl TryFrom<PrimitiveValue> for $type {
            type Error = PrimitiveValue;

            fn try_from(x: PrimitiveValue) -> Result<Self, Self::Error> {
                <$type as PrimitiveValueLike>::try_from_primitive_value(x)
            }
        }
    )+};
}

impl_conv!(bool char f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize);

impl PrimitiveValueLike for () {
    fn into_primitive_value(self) -> PrimitiveValue {
        PrimitiveValue::Unit
    }

    fn try_from_primitive_value(primitive_value: PrimitiveValue) -> Result<Self, PrimitiveValue> {
        match primitive_value {
            PrimitiveValue::Unit => Ok(()),
            x => Err(x),
        }
    }
}

impl From<()> for PrimitiveValue {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl TryFrom<PrimitiveValue> for () {
    type Error = PrimitiveValue;

    fn try_from(x: PrimitiveValue) -> Result<Self, Self::Error> {
        PrimitiveValueLike::try_from_primitive_value(x)
    }
}

/// Represents primitive value types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
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

    /// Constructs a primitive value type from a Rust-based type string similar
    /// to what you would find from `std::any::type_name`
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{PrimitiveValueType as PVT, NumberType as NT};
    ///
    /// assert_eq!(
    ///     PVT::from_type_name("bool").unwrap(),
    ///     PVT::Bool,
    /// );
    ///
    /// assert_eq!(
    ///     PVT::from_type_name("char").unwrap(),
    ///     PVT::Char,
    /// );
    ///
    /// assert_eq!(
    ///     PVT::from_type_name("u8").unwrap(),
    ///     PVT::Number(NT::U8),
    /// );
    ///
    /// assert_eq!(
    ///     PVT::from_type_name("()").unwrap(),
    ///     PVT::Unit,
    /// );
    /// ```
    pub fn from_type_name(tname: &str) -> Result<Self, ParseError> {
        use std::str::FromStr;

        // Translate any Rust-specific types to our custom format, passing
        // anything that is the same to our FromStr implementation
        match tname {
            "()" => Self::from_str("unit"),
            x => Self::from_str(x),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_value_like_can_convert_number_like_to_primitive_value() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_primitive_value_to_number_like() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_bool_to_primitive_value() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_primitive_value_to_bool() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_char_to_primitive_value() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_primitive_value_to_char() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_unit_to_primitive_value() {
        todo!()
    }

    #[test]
    fn primitive_value_like_can_convert_primitive_value_to_unit() {
        todo!()
    }
}
