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
pub enum Primitive {
    Bool(bool),
    Char(char),
    Number(Number),
    Unit,
}

impl Default for PrimitiveType {
    /// Returns default primitive value type of unit
    fn default() -> Self {
        Self::Unit
    }
}

impl Primitive {
    /// Returns true if this value is of the specified type
    #[inline]
    pub fn is_type(&self, r#type: PrimitiveType) -> bool {
        self.to_type() == r#type
    }

    /// Returns the type of this value
    #[inline]
    pub fn to_type(&self) -> PrimitiveType {
        PrimitiveType::from(self)
    }

    /// Returns true if this value and the other value are of the same type
    #[inline]
    pub fn has_same_type(&self, other: &Primitive) -> bool {
        self.to_type() == other.to_type()
    }
}

impl Hash for Primitive {
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
impl Eq for Primitive {}

impl PartialEq for Primitive {
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

impl PartialOrd for Primitive {
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

/// Represents some data that can be converted to and from a [`Primitive`]
pub trait PrimitiveLike: Sized {
    /// Consumes this data, converting it into an abstract [`Primitive`]
    fn into_primitive(self) -> Primitive;

    /// Attempts to convert an abstract [`Primitive`] into this data, returning
    /// the owned value back if unable to convert
    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive>;
}

impl PrimitiveLike for Primitive {
    fn into_primitive(self) -> Primitive {
        self
    }

    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive> {
        Ok(primitive)
    }
}

impl PrimitiveLike for bool {
    fn into_primitive(self) -> Primitive {
        Primitive::Bool(self)
    }

    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive> {
        match primitive {
            Primitive::Bool(x) => Ok(x),
            x => Err(x),
        }
    }
}

impl PrimitiveLike for char {
    fn into_primitive(self) -> Primitive {
        Primitive::Char(self)
    }

    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive> {
        match primitive {
            Primitive::Char(x) => Ok(x),
            x => Err(x),
        }
    }
}

impl<T: NumberLike> PrimitiveLike for T {
    fn into_primitive(self) -> Primitive {
        Primitive::Number(self.into_number())
    }

    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive> {
        match primitive {
            Primitive::Number(x) => T::try_from_number(x).map_err(Primitive::Number),
            x => Err(x),
        }
    }
}

macro_rules! impl_conv {
    ($($type:ty)+) => {$(
        impl From<$type> for Primitive {
            fn from(x: $type) -> Self {
                <$type as PrimitiveLike>::into_primitive(x)
            }
        }

        impl TryFrom<Primitive> for $type {
            type Error = Primitive;

            fn try_from(x: Primitive) -> Result<Self, Self::Error> {
                <$type as PrimitiveLike>::try_from_primitive(x)
            }
        }
    )+};
}

impl_conv!(bool char f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize);

impl PrimitiveLike for () {
    fn into_primitive(self) -> Primitive {
        Primitive::Unit
    }

    fn try_from_primitive(primitive: Primitive) -> Result<Self, Primitive> {
        match primitive {
            Primitive::Unit => Ok(()),
            x => Err(x),
        }
    }
}

impl From<()> for Primitive {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl TryFrom<Primitive> for () {
    type Error = Primitive;

    fn try_from(x: Primitive) -> Result<Self, Self::Error> {
        PrimitiveLike::try_from_primitive(x)
    }
}

/// Represents primitive value types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveType {
    Bool,
    Char,
    Number(NumberType),
    Unit,
}

impl PrimitiveType {
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
    /// use entity::{PrimitiveType as PVT, NumberType as NT};
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

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Char => write!(f, "char"),
            Self::Number(t) => write!(f, "number:{}", t),
            Self::Unit => write!(f, "unit"),
        }
    }
}

impl From<Primitive> for PrimitiveType {
    fn from(v: Primitive) -> Self {
        Self::from(&v)
    }
}

impl<'a> From<&'a Primitive> for PrimitiveType {
    fn from(v: &'a Primitive) -> Self {
        match v {
            Primitive::Bool(_) => Self::Bool,
            Primitive::Char(_) => Self::Char,
            Primitive::Number(x) => Self::Number(x.to_type()),
            Primitive::Unit => Self::Unit,
        }
    }
}

impl std::str::FromStr for PrimitiveType {
    type Err = ParseError;

    /// Parses a primitive value type
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{PrimitiveType as PVT, NumberType as NT};
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
    fn primitive_like_can_convert_number_like_to_primitive() {
        assert!(matches!(
            1u8.into_primitive(),
            Primitive::Number(Number::U8(1)),
        ));
    }

    #[test]
    fn primitive_like_can_convert_primitive_to_number_like() {
        assert!(matches!(
            Number::try_from_primitive(Primitive::Number(Number::U8(1))),
            Ok(Number::U8(1)),
        ));

        assert!(matches!(
            Number::try_from_primitive(Primitive::Char('c')),
            Err(Primitive::Char('c')),
        ));
    }

    #[test]
    fn primitive_like_can_convert_bool_to_primitive() {
        assert!(matches!(true.into_primitive(), Primitive::Bool(true),));
    }

    #[test]
    fn primitive_like_can_convert_primitive_to_bool() {
        assert!(matches!(
            bool::try_from_primitive(Primitive::Bool(true)),
            Ok(true)
        ));

        assert!(matches!(
            bool::try_from_primitive(Primitive::Char('c')),
            Err(Primitive::Char('c')),
        ));
    }

    #[test]
    fn primitive_like_can_convert_char_to_primitive() {
        assert!(matches!('c'.into_primitive(), Primitive::Char('c')));
    }

    #[test]
    fn primitive_like_can_convert_primitive_to_char() {
        assert!(matches!(
            char::try_from_primitive(Primitive::Char('c')),
            Ok('c')
        ));

        assert!(matches!(
            char::try_from_primitive(Primitive::Bool(true)),
            Err(Primitive::Bool(true)),
        ));
    }

    #[test]
    fn primitive_like_can_convert_unit_to_primitive() {
        assert!(matches!(().into_primitive(), Primitive::Unit));
    }

    #[test]
    fn primitive_like_can_convert_primitive_to_unit() {
        assert!(matches!(<()>::try_from_primitive(Primitive::Unit), Ok(())));

        assert!(matches!(
            <()>::try_from_primitive(Primitive::Bool(true)),
            Err(Primitive::Bool(true)),
        ));
    }
}
