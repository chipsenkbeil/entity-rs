use doc_comment::doc_comment;
use paste::paste;
use std::{
    cmp::Ordering,
    convert::TryFrom,
    hash::{Hash, Hasher},
};
use strum::{Display, EnumDiscriminants, EnumString};

/// Represents a generic number that maintains an internal Rust representation
/// of the actual number
#[derive(Copy, Clone, Debug, EnumDiscriminants)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(Display, EnumString))]
#[strum_discriminants(name(NumberType), strum(serialize_all = "snake_case"))]
#[cfg_attr(
    feature = "serde-1",
    strum_discriminants(derive(serde::Serialize, serde::Deserialize))
)]
pub enum Number {
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
    Usize(usize),
}

macro_rules! impl_cast {
    ($type:ty) => {
        paste! {
            doc_comment! {
                concat!(
                    "Naive casting of number's inner representation to ",
                    stringify!($type),
                    "by performing `x as ", stringify!($type), "`",
                ),
                #[inline]
                pub fn [<to_ $type>](&self) -> $type {
                    match self {
                        Self::F32(x) => *x as $type,
                        Self::F64(x) => *x as $type,
                        Self::I128(x) => *x as $type,
                        Self::I16(x) => *x as $type,
                        Self::I32(x) => *x as $type,
                        Self::I64(x) => *x as $type,
                        Self::I8(x) => *x as $type,
                        Self::Isize(x) => *x as $type,
                        Self::U128(x) => *x as $type,
                        Self::U16(x) => *x as $type,
                        Self::U32(x) => *x as $type,
                        Self::U64(x) => *x as $type,
                        Self::U8(x) => *x as $type,
                        Self::Usize(x) => *x as $type,
                    }
                }
            }
        }
    };
}

impl Number {
    /// Returns an indicator of the sign (negative, zero, positive)
    /// of this number
    ///
    /// ```
    /// use entity::{Number, NumberSign};
    ///
    /// assert_eq!(Number::from(0).sign(), NumberSign::Zero);
    /// assert_eq!(Number::from(99).sign(), NumberSign::Positive);
    /// assert_eq!(Number::from(-99).sign(), NumberSign::Negative);
    /// ```
    pub fn sign(&self) -> NumberSign {
        if self.is_negative() {
            NumberSign::Negative
        } else if self.is_positive() {
            NumberSign::Positive
        } else {
            NumberSign::Zero
        }
    }

    /// Returns true if number is zero (not negative or positive)
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(0).is_zero());
    /// assert!(!Number::from(1).is_zero());
    /// assert!(!Number::from(-1).is_zero());
    /// ```
    #[inline]
    pub fn is_zero(&self) -> bool {
        match self {
            Self::F32(x) => *x == 0.0,
            Self::F64(x) => *x == 0.0,
            Self::I128(x) => *x == 0,
            Self::I16(x) => *x == 0,
            Self::I32(x) => *x == 0,
            Self::I64(x) => *x == 0,
            Self::I8(x) => *x == 0,
            Self::Isize(x) => *x == 0,
            Self::U128(x) => *x == 0,
            Self::U16(x) => *x == 0,
            Self::U32(x) => *x == 0,
            Self::U64(x) => *x == 0,
            Self::U8(x) => *x == 0,
            Self::Usize(x) => *x == 0,
        }
    }

    /// Returns true if number is positive (not zero or negative)
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(1).is_positive());
    /// assert!(!Number::from(0).is_positive());
    /// assert!(!Number::from(-1).is_positive());
    /// ```
    #[inline]
    pub fn is_positive(&self) -> bool {
        !self.is_zero() && !self.is_negative()
    }

    /// Returns true if number is negative (not zero or positive)
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(-1).is_negative());
    /// assert!(!Number::from(0).is_negative());
    /// assert!(!Number::from(1).is_negative());
    /// ```
    #[inline]
    pub fn is_negative(&self) -> bool {
        match self {
            Self::F32(x) => x.is_normal() && x.is_sign_negative(),
            Self::F64(x) => x.is_normal() && x.is_sign_negative(),
            Self::I128(x) => x.is_negative(),
            Self::I16(x) => x.is_negative(),
            Self::I32(x) => x.is_negative(),
            Self::I64(x) => x.is_negative(),
            Self::I8(x) => x.is_negative(),
            Self::Isize(x) => x.is_negative(),
            _ => false,
        }
    }

    /// Returns true if number is a signed integer
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(3isize).is_signed());
    /// assert!(!Number::from(3usize).is_signed());
    /// ```
    #[inline]
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::I128(_)
            | Self::I16(_)
            | Self::I32(_)
            | Self::I64(_)
            | Self::I8(_)
            | Self::Isize(_)
        )
    }

    /// Returns true if number is an unsigned integer
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(3usize).is_unsigned());
    /// assert!(!Number::from(3isize).is_unsigned());
    /// ```
    #[inline]
    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Self::U128(_)
            | Self::U16(_)
            | Self::U32(_)
            | Self::U64(_)
            | Self::U8(_)
            | Self::Usize(_)
        )
    }

    /// Returns true if number is a float
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(3f32).is_float());
    /// assert!(!Number::from(3usize).is_float());
    /// ```
    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32(_) | Self::F64(_))
    }

    /// Returns true if number is a float with a non-zero fractional part
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(3.1).has_nonzero_fraction());
    /// assert!(!Number::from(3.0).has_nonzero_fraction());
    /// ```
    pub fn has_nonzero_fraction(&self) -> bool {
        match self {
            Self::F32(x) => x.fract().is_normal(),
            Self::F64(x) => x.fract().is_normal(),
            _ => false,
        }
    }

    /// Returns true if number is neither zero, infinite, subnormal, or NaN
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::Number;
    ///
    /// assert!(Number::from(1).is_normal());
    /// assert!(Number::from(0.1).is_normal());
    /// assert!(!Number::from(0).is_normal());
    /// assert!(!Number::from(f64::NAN).is_normal());
    /// assert!(!Number::from(f64::INFINITY).is_normal());
    /// assert!(!Number::from(f64::NEG_INFINITY).is_normal());
    /// assert!(!Number::from(1.0e-308_f64).is_normal());
    /// ```
    #[inline]
    pub fn is_normal(&self) -> bool {
        match self {
            Self::F32(x) => x.is_normal(),
            Self::F64(x) => x.is_normal(),
            _ => !self.is_zero(),
        }
    }

    /// Returns a conversion of the underlying number to an absolute version
    /// of itself
    #[inline]
    pub fn to_absolute(&self) -> Self {
        match self {
            Self::F32(x) => Self::F32(x.abs()),
            Self::F64(x) => Self::F64(x.abs()),
            Self::I128(x) => Self::I128(x.abs()),
            Self::I16(x) => Self::I16(x.abs()),
            Self::I32(x) => Self::I32(x.abs()),
            Self::I64(x) => Self::I64(x.abs()),
            Self::I8(x) => Self::I8(x.abs()),
            Self::Isize(x) => Self::Isize(x.abs()),
            Self::U128(x) => Self::U128(*x),
            Self::U16(x) => Self::U16(*x),
            Self::U32(x) => Self::U32(*x),
            Self::U64(x) => Self::U64(*x),
            Self::U8(x) => Self::U8(*x),
            Self::Usize(x) => Self::Usize(*x),
        }
    }

    impl_cast!(f64);
    impl_cast!(f32);
    impl_cast!(isize);
    impl_cast!(i128);
    impl_cast!(i64);
    impl_cast!(i32);
    impl_cast!(i16);
    impl_cast!(i8);
    impl_cast!(usize);
    impl_cast!(u128);
    impl_cast!(u64);
    impl_cast!(u32);
    impl_cast!(u16);
    impl_cast!(u8);

    /// Converts into type of number
    #[inline]
    pub fn to_type(&self) -> NumberType {
        self.into()
    }
}

/// Represents some data that can be converted to and from a [`Number`]
pub trait NumberLike: Sized {
    /// Consumes this data, converting it into an abstract [`Number`]
    fn into_number(self) -> Number;

    /// Attempts to convert an abstract [`Number`] into this data, returning
    /// the owned value back if unable to convert
    fn try_from_number(number: Number) -> Result<Self, Number>;
}

impl NumberLike for Number {
    fn into_number(self) -> Number {
        self
    }

    fn try_from_number(number: Number) -> Result<Self, Number> {
        Ok(number)
    }
}

macro_rules! try_from_both_bounded {
    ($val:ident, $variant:ident, $source:ty, $target:ty) => {{
        let min = Self::MIN as $source;
        let max = Self::MAX as $source;
        if $val < min || $val > max {
            Err(Number::$variant($val))
        } else {
            Ok($val as $target)
        }
    }};
}

impl NumberLike for f32 {
    fn into_number(self) -> Number {
        Number::F32(self)
    }

    fn try_from_number(number: Number) -> Result<Self, Number> {
        match number {
            Number::F32(x) => Ok(x),
            Number::F64(x) => try_from_both_bounded!(x, F64, f64, f32),
            Number::I128(x) => try_from_both_bounded!(x, I128, i128, f32),
            Number::I16(x) => Ok(Self::from(x)),
            Number::I32(x) => try_from_both_bounded!(x, I32, i32, f32),
            Number::I64(x) => try_from_both_bounded!(x, I64, i64, f32),
            Number::I8(x) => Ok(Self::from(x)),
            Number::Isize(x) => try_from_both_bounded!(x, Isize, isize, f32),
            Number::U128(x) => try_from_both_bounded!(x, U128, u128, f32),
            Number::U16(x) => Ok(Self::from(x)),
            Number::U32(x) => try_from_both_bounded!(x, U32, u32, f32),
            Number::U64(x) => try_from_both_bounded!(x, U64, u64, f32),
            Number::U8(x) => Ok(Self::from(x)),
            Number::Usize(x) => try_from_both_bounded!(x, Usize, usize, f32),
        }
    }
}

impl From<f32> for Number {
    fn from(x: f32) -> Self {
        <f32 as NumberLike>::into_number(x)
    }
}

impl TryFrom<Number> for f32 {
    type Error = Number;

    fn try_from(x: Number) -> Result<Self, Self::Error> {
        <f32 as NumberLike>::try_from_number(x)
    }
}

impl NumberLike for f64 {
    fn into_number(self) -> Number {
        Number::F64(self)
    }

    fn try_from_number(number: Number) -> Result<Self, Number> {
        match number {
            Number::F32(x) => Ok(Self::from(x)),
            Number::F64(x) => Ok(x),
            Number::I128(x) => try_from_both_bounded!(x, I128, i128, f64),
            Number::I16(x) => Ok(Self::from(x)),
            Number::I32(x) => Ok(Self::from(x)),
            Number::I64(x) => try_from_both_bounded!(x, I64, i64, f64),
            Number::I8(x) => Ok(Self::from(x)),
            Number::Isize(x) => try_from_both_bounded!(x, Isize, isize, f64),
            Number::U128(x) => try_from_both_bounded!(x, U128, u128, f64),
            Number::U16(x) => Ok(Self::from(x)),
            Number::U32(x) => Ok(Self::from(x)),
            Number::U64(x) => try_from_both_bounded!(x, U64, u64, f64),
            Number::U8(x) => Ok(Self::from(x)),
            Number::Usize(x) => try_from_both_bounded!(x, Usize, usize, f64),
        }
    }
}

impl From<f64> for Number {
    fn from(x: f64) -> Self {
        <f64 as NumberLike>::into_number(x)
    }
}

impl TryFrom<Number> for f64 {
    type Error = Number;

    fn try_from(x: Number) -> Result<Self, Self::Error> {
        <f64 as NumberLike>::try_from_number(x)
    }
}

macro_rules! impl_number_like {
    ($type:ty, $variant:ident) => {
        impl NumberLike for $type {
            fn into_number(self) -> Number {
                Number::$variant(self)
            }

            /// Attempts to convert from generic number, succeeding if within
            /// finite bounds of target type, otherwise failing and returning
            /// back ownership of generic number
            fn try_from_number(number: Number) -> Result<Self, Number> {
                match number {
                    Number::F32(x) => try_from_both_bounded!(x, F32, f32, $type),
                    Number::F64(x) => try_from_both_bounded!(x, F64, f64, $type),
                    Number::I128(x) => <$type>::try_from(x).map_err(|_| Number::I128(x)),
                    Number::I16(x) => <$type>::try_from(x).map_err(|_| Number::I16(x)),
                    Number::I32(x) => <$type>::try_from(x).map_err(|_| Number::I32(x)),
                    Number::I64(x) => <$type>::try_from(x).map_err(|_| Number::I64(x)),
                    Number::I8(x) => <$type>::try_from(x).map_err(|_| Number::I8(x)),
                    Number::Isize(x) => <$type>::try_from(x).map_err(|_| Number::Isize(x)),
                    Number::U128(x) => <$type>::try_from(x).map_err(|_| Number::U128(x)),
                    Number::U16(x) => <$type>::try_from(x).map_err(|_| Number::U16(x)),
                    Number::U32(x) => <$type>::try_from(x).map_err(|_| Number::U32(x)),
                    Number::U64(x) => <$type>::try_from(x).map_err(|_| Number::U64(x)),
                    Number::U8(x) => <$type>::try_from(x).map_err(|_| Number::U8(x)),
                    Number::Usize(x) => <$type>::try_from(x).map_err(|_| Number::Usize(x)),
                }
            }
        }

        impl From<$type> for Number {
            fn from(x: $type) -> Self {
                <$type as NumberLike>::into_number(x)
            }
        }

        impl TryFrom<Number> for $type {
            type Error = Number;

            fn try_from(x: Number) -> Result<Self, Self::Error> {
                <$type as NumberLike>::try_from_number(x)
            }
        }
    };
}

impl_number_like!(isize, Isize);
impl_number_like!(i128, I128);
impl_number_like!(i64, I64);
impl_number_like!(i32, I32);
impl_number_like!(i16, I16);
impl_number_like!(i8, I8);
impl_number_like!(usize, Usize);
impl_number_like!(u128, U128);
impl_number_like!(u64, U64);
impl_number_like!(u32, U32);
impl_number_like!(u16, U16);
impl_number_like!(u8, U8);

/// Represents the sign of a number
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum NumberSign {
    Positive,
    Negative,
    Zero,
}

impl NumberSign {
    /// Returns a signed 8-bit number representing the sign
    ///
    /// * 0 if the number is zero
    /// * 1 if the number is positive
    /// * -1 if the number is negative
    ///
    /// ```
    /// use entity::NumberSign;
    ///
    /// assert_eq!(NumberSign::Zero.to_i8(), 0);
    /// assert_eq!(NumberSign::Positive.to_i8(), 1);
    /// assert_eq!(NumberSign::Negative.to_i8(), -1);
    /// ```
    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
            Self::Zero => 0,
        }
    }
}
impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
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
            Self::Usize(x) => x.hash(state),
        }
    }
}

impl Eq for Number {}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match self.partial_cmp(other) {
            Some(o) => match o {
                Ordering::Equal => true,
                Ordering::Greater | Ordering::Less => false,
            },
            None => false,
        }
    }
}

impl PartialOrd for Number {
    /// Compares two numbers if possible. If either number is not zero and
    /// not normal as defined by Rust's specification, None is returned.
    ///
    /// self sign | other sign | situation
    /// ----------|------------|----------
    /// negative  | negative   | does comparison (less/equal/greater)
    /// negative  | positive   | less than
    /// negative  | zero       | less than
    /// positive  | negative   | greater than
    /// positive  | positive   | does comparison (less/equal/greater)
    /// positive  | zero       | greater than
    /// zero      | negative   | greater than
    /// zero      | positive   | less than
    /// zero      | zero       | equal
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // If we have a number that is not comparable, we return none
        if !self.is_normal() && !self.is_zero() || !other.is_normal() && !other.is_zero() {
            return None;
        }

        match (self.sign(), other.sign()) {
            (NumberSign::Negative, NumberSign::Negative) => Some(
                other
                    .to_absolute()
                    .to_u128()
                    .cmp(&self.to_absolute().to_u128()),
            ),
            (NumberSign::Negative, NumberSign::Positive) => Some(Ordering::Less),
            (NumberSign::Negative, NumberSign::Zero) => Some(Ordering::Less),
            (NumberSign::Positive, NumberSign::Negative) => Some(Ordering::Greater),
            (NumberSign::Positive, NumberSign::Positive) => {
                Some(self.to_u128().cmp(&other.to_u128()))
            }
            (NumberSign::Positive, NumberSign::Zero) => Some(Ordering::Greater),
            (NumberSign::Zero, NumberSign::Negative) => Some(Ordering::Greater),
            (NumberSign::Zero, NumberSign::Positive) => Some(Ordering::Less),
            (NumberSign::Zero, NumberSign::Zero) => Some(Ordering::Equal),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;

    macro_rules! check {
        (eq $value:expr, $type:ty; $fname:ident, $x:literal) => {{
            let number = Number::from(($x) as $type);
            let result = number.clone().$fname();
            assert_eq!(
                result, $value,
                "Calling {} on {:?}, expected {:?}, but got {:?}",
                stringify!($fname), number, $value, result,
            );
        }};
        (not eq $value:expr, $type:ty; $fname:ident, $x:literal) => {
            let number = Number::from(($x) as $type);
            assert_ne!(
                number.clone().$fname(), $value,
                "Calling {} on {:?}, unexpectedly got {:?}",
                stringify!($fname), number, $value,
            );
        };
        ($type:ty; $fname:ident, $x:literal) => {
            check!(eq true, $type; $fname, $x);
        };
        (not $type:ty; $fname:ident, $x:literal) => {
            check!(eq false, $type; $fname, $x);
        };
        ($type:ty; $fname:ident, $x:literal, $($y:literal),+) => {{
            check!($type; $fname, $x);
            check!($type; $fname, $($y),+);
        }};
        (not $type:ty; $fname:ident, $x:literal, $($y:literal),+) => {{
            check!(not $type; $fname, $x);
            check!(not $type; $fname, $($y),+);
        }};
        (eq $value:expr, $type:ty; $fname:ident, $x:literal, $($y:literal),+) => {{
            check!(eq $value, $type; $fname, $x);
            check!(eq $value, $type; $fname, $($y),+);
        }};
        (not eq $value:expr, $type:ty; $fname:ident, $x:literal, $($y:literal),+) => {{
            check!(not eq $value, $type; $fname, $x);
            check!(not eq $value, $type; $fname, $($y),+);
        }};
        ($type:ty; $fname:ident, $(eq $value:expr, )?$($x:literal),+) => {{
            check!($(eq $value, )?$type; $fname, $($x),+);
        }};
        ($a:ty, $($b:ty),+; $fname:ident, $(eq $value:expr, )?$($x:literal),+) => {{
            check!($a; $fname, $(eq $value, )?$($x),+);
            check!($($b),+; $fname, $(eq $value, )?$($x),+);
        }};
        ($type:ty; $fname:ident, $(eq $value:expr, )?not $($x:literal),+) => {{
            check!(not $(eq $value, )?$type; $fname, $($x),+);
        }};
        ($a:ty, $($b:ty),+; $fname:ident, $(eq $value:expr, )?not $($x:literal),+) => {{
            check!($a; $fname, $(eq $value, )?not $($x),+);
            check!($($b),+; $fname, $(eq $value, )?not $($x),+);
        }};
        ($type:ty; $fname:ident, $(eq $value:expr, )?$($x:literal),+, not $($y:literal),+) => {{
            check!($type; $fname, $(eq $value, )?$($x),+);
            check!($type; $fname, $(eq $value, )?not $($y),+);
        }};
        ($a:ty, $($b:ty),+; $fname:ident, $(eq $value:expr, )?$($x:literal),+, not $($y:literal),+) => {{
            check!($a; $fname, $(eq $value, )?$($x),+, not $($y),+);
            check!($($b),+; $fname, $(eq $value, )?$($x),+, not $($y),+);
        }};
    }

    #[test]
    fn partial_cmp_should_return_none_if_either_number_is_nan() {
        let a = Number::from(1);
        let b = Number::from(f32::NAN);
        assert_eq!(a.partial_cmp(&b), None);

        let a = Number::from(f32::NAN);
        let b = Number::from(1);
        assert_eq!(a.partial_cmp(&b), None);
    }

    #[test]
    fn partial_cmp_should_return_none_if_either_number_is_infinite() {
        let a = Number::from(1);
        let b = Number::from(f32::INFINITY);
        assert_eq!(a.partial_cmp(&b), None);

        let a = Number::from(f32::INFINITY);
        let b = Number::from(1);
        assert_eq!(a.partial_cmp(&b), None);

        let a = Number::from(1);
        let b = Number::from(f32::NEG_INFINITY);
        assert_eq!(a.partial_cmp(&b), None);

        let a = Number::from(f32::NEG_INFINITY);
        let b = Number::from(1);
        assert_eq!(a.partial_cmp(&b), None);
    }

    #[test]
    fn partial_cmp_should_return_none_if_either_number_is_subnormal() {
        let a = Number::from(1);
        let b = Number::from(1.0e-40_f32);
        assert_eq!(a.partial_cmp(&b), None);

        let a = Number::from(1.0e-40_f32);
        let b = Number::from(1);
        assert_eq!(a.partial_cmp(&b), None);
    }

    #[test]
    fn partial_cmp_should_perform_cmp_if_both_numbers_negative() {
        let a = Number::from(-2isize);
        let b = Number::from(-1i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));

        let a = Number::from(-1isize);
        let b = Number::from(-1i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));

        let a = Number::from(-1isize);
        let b = Number::from(-2i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn partial_cmp_should_return_less_than_if_negative_and_other_positive() {
        let a = Number::from(-1isize);
        let b = Number::from(1usize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn partial_cmp_should_return_less_than_if_negative_and_other_zero() {
        let a = Number::from(-1isize);
        let b = Number::from(0usize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn partial_cmp_should_return_greater_than_if_positive_and_other_negative() {
        let a = Number::from(1usize);
        let b = Number::from(-1isize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn partial_cmp_should_perform_cmp_if_both_numbers_positive() {
        let a = Number::from(2isize);
        let b = Number::from(1i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));

        let a = Number::from(1isize);
        let b = Number::from(1i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));

        let a = Number::from(1isize);
        let b = Number::from(2i8);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn partial_cmp_should_return_greater_than_if_positive_and_other_zero() {
        let a = Number::from(1isize);
        let b = Number::from(0usize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn partial_cmp_should_return_greater_than_if_zero_and_other_negative() {
        let a = Number::from(0usize);
        let b = Number::from(-1isize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn partial_cmp_should_return_less_than_if_zero_and_other_positive() {
        let a = Number::from(0usize);
        let b = Number::from(1isize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn partial_cmp_should_return_equal_if_both_zero() {
        let a = Number::from(0usize);
        let b = Number::from(0isize);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    }

    #[test]
    fn sign_should_return_positive_if_inner_value_is_positive() {
        check!(f32, f64; sign, eq NumberSign::Positive, 0.1, not 0.0, -0.1);
        check!(i128, i64, i32, i16, i8, isize; sign, eq NumberSign::Positive, 1, not 0, -1);
        check!(u128, u64, u32, u16, u8, usize; sign, eq NumberSign::Positive, 1, not 0);
    }

    #[test]
    fn sign_should_return_negative_if_inner_value_is_negative() {
        check!(f32, f64; sign, eq NumberSign::Negative, -0.1, not 0.0, 0.1);
        check!(i128, i64, i32, i16, i8, isize; sign, eq NumberSign::Negative, -1, not 0, 1);
        check!(u128, u64, u32, u16, u8, usize; sign, eq NumberSign::Negative, not 0, 1);
    }

    #[test]
    fn sign_should_return_zero_if_inner_value_is_zero() {
        check!(f32, f64; sign, eq NumberSign::Zero, 0.0, not -0.1, 0.1);
        check!(i128, i64, i32, i16, i8, isize; sign, eq NumberSign::Zero, 0, not -1, 1);
        check!(u128, u64, u32, u16, u8, usize; sign, eq NumberSign::Zero, 0, not 1);
    }

    #[test]
    fn is_zero_should_return_true_if_inner_value_is_zero() {
        check!(f32, f64; is_zero, 0.0, not -0.1, 0.1);
        check!(i128, i64, i32, i16, i8, isize; is_zero, 0, not -1, 1);
        check!(u128, u64, u32, u16, u8, usize; is_zero, 0, not 1);
    }

    #[test]
    fn is_positive_should_return_true_if_inner_value_is_positive() {
        check!(f32, f64; is_positive, 0.1, not 0.0, -0.1);
        check!(i128, i64, i32, i16, i8, isize; is_positive, 1, not -1, 0);
        check!(u128, u64, u32, u16, u8, usize; is_positive, 1, not 0);
    }

    #[test]
    fn is_negative_should_return_true_if_inner_value_is_negative() {
        check!(f32, f64; is_negative, -0.1, not 0.0, 0.1);
        check!(i128, i64, i32, i16, i8, isize; is_negative, -1, not 0, 1);
        check!(u128, u64, u32, u16, u8, usize; is_negative, not 0, 1);
    }

    #[test]
    fn is_signed_should_return_true_if_inner_value_is_a_signed_integer() {
        check!(f32, f64; is_signed, not -0.1, 0.0, 0.1);
        check!(i128, i64, i32, i16, i8, isize; is_signed, -1, 0, 1);
        check!(u128, u64, u32, u16, u8, usize; is_signed, not 0, 1);
    }

    #[test]
    fn is_unsigned_should_return_true_if_inner_value_is_an_unsigned_integer() {
        check!(f32, f64; is_unsigned, not -0.1, 0.0, 0.1);
        check!(i128, i64, i32, i16, i8, isize; is_unsigned, not -1, 0, 1);
        check!(u128, u64, u32, u16, u8, usize; is_unsigned, 0, 1);
    }

    #[test]
    fn is_float_should_return_true_if_inner_value_is_a_float() {
        check!(f32, f64; is_float, -0.1, 0.0, 0.1);
        check!(i128, i64, i32, i16, i8, isize; is_float, not -1, 0, 1);
        check!(u128, u64, u32, u16, u8, usize; is_float, not 0, 1);
    }

    #[test]
    fn has_nonzero_fraction_should_return_true_if_float_with_fractional_part() {
        check!(f32, f64; has_nonzero_fraction, -0.1, -1.1, 1.1, 0.1, not 0.0, 1.0, -1.0);
        check!(i128, i64, i32, i16, i8, isize; has_nonzero_fraction, not -1, 0, 1);
        check!(u128, u64, u32, u16, u8, usize; has_nonzero_fraction, not 0, 1);
    }

    #[test]
    fn is_normal_should_return_true_if_inner_value_is_not_zero_infinite_nan_or_subnormal() {
        check!(f32; is_normal, -0.1, -1.1, 1.1, 0.1, not 0.0, 1.0e-40_f32);
        assert!(!Number::from(f32::NAN).is_normal());
        assert!(!Number::from(f32::INFINITY).is_normal());
        assert!(!Number::from(f32::NEG_INFINITY).is_normal());

        check!(f64; is_normal, -0.1, -1.1, 1.1, 0.1, not 0.0, 1.0e-308_f64);
        assert!(!Number::from(f64::NAN).is_normal());
        assert!(!Number::from(f64::INFINITY).is_normal());
        assert!(!Number::from(f64::NEG_INFINITY).is_normal());

        check!(i128, i64, i32, i16, i8, isize; is_normal, -1, 1, not 0);
        check!(u128, u64, u32, u16, u8, usize; is_normal, 1, not 0);
    }

    #[test]
    fn to_absolute_should_return_new_number_converted_to_absolute_value() {
        macro_rules! check_abs_match {
            ($variant:ident; $x:literal, $y:literal) => {
                match (Number::$variant($x).to_absolute(), Number::$variant($y)) {
                    (Number::$variant(x), Number::$variant(y)) => {
                        assert_eq!(x, y, "Expected abs({}) -> {}, but was {}", $x, $y, x);
                    }
                    x => panic!("Unexpected comparison: {:?}", x),
                }
            };
            ($a:ident, $($b:ident),+; $x:literal, $y:literal) => {
                check_abs_match!($a; $x, $y);
                check_abs_match!($($b),+; $x, $y);
            }
        }

        check_abs_match!(F32, F64; -0.1, 0.1);
        check_abs_match!(F32, F64; 0.0, 0.0);
        check_abs_match!(F32, F64; 0.1, 0.1);

        check_abs_match!(I128, I64, I32, I16, I8, Isize; -1, 1);
        check_abs_match!(I128, I64, I32, I16, I8, Isize; 0, 0);
        check_abs_match!(I128, I64, I32, I16, I8, Isize; 1, 1);

        check_abs_match!(U128, U64, U32, U16, U8, Usize; 0, 0);
        check_abs_match!(U128, U64, U32, U16, U8, Usize; 1, 1);
    }

    #[test]
    fn number_like_can_convert_number_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_f32_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_f32() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_f64_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_f64() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_isize_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_isize() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_i128_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_i128() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_i64_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_i64() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_i32_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_i32() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_i16_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_i16() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_i8_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_i8() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_usize_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_usize() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_u128_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_u128() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_u64_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_u64() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_u32_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_u32() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_u16_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_u16() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_u8_to_number() {
        todo!()
    }

    #[test]
    fn number_like_can_convert_number_to_u8() {
        todo!()
    }
}
