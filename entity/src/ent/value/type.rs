use super::{NumberType, PrimitiveValueType, Value};
use strum::ParseError;

/// Represents value types (primitive or complex). Assumes that complex
/// types will contain the same inner type and does not vary
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ValueType {
    List(Box<ValueType>),
    Map(Box<ValueType>),
    Optional(Box<ValueType>),
    Primitive(PrimitiveValueType),
    Text,
    Custom,
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

    /// Constructs a value type from a Rust-based type string similar to what
    /// you would find from `std::any::type_name`
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{ValueType as VT, PrimitiveValueType as PVT, NumberType as NT};
    ///
    /// assert_eq!(
    ///     VT::from_type_name("u8").expect("one"),
    ///     VT::Primitive(PVT::Number(NT::U8)),
    /// );
    ///
    /// assert_eq!(
    ///     VT::from_type_name("std::vec::Vec<std::string::String>").expect("two"),
    ///     VT::List(Box::from(VT::Text)),
    /// );
    ///
    /// assert_eq!(
    ///     VT::from_type_name("Vec<Option<u8>>").expect("three"),
    ///     VT::List(Box::from(VT::Optional(Box::from(VT::Primitive(PVT::Number(NT::U8)))))),
    /// );
    ///
    /// assert_eq!(
    ///     VT::from_type_name("HashMap<String, u8>").expect("four"),
    ///     VT::Map(Box::from(VT::Primitive(PVT::Number(NT::U8)))),
    /// );
    /// ```
    pub fn from_type_name(name: &str) -> Result<Self, ParseError> {
        if name.is_empty() {
            return Err(ParseError::VariantNotFound);
        }

        // Split based on the start of a generic in the form of Outer<Inner>
        let mut tokens = name.split(|c| c == '<');
        let maybe_outer_str = tokens.next();
        let inner_str = {
            let mut x = tokens.collect::<Vec<&str>>().join("<");
            if x.ends_with('>') {
                x.pop();
            }
            x
        };

        // Get the outer type based on an equivalent rust type
        //
        // * HashMap | BTreeMap -> Map
        // * Vec | VecDeque | LinkedList | HashSet | BTreeSet | BinaryHeap -> List
        // * Option -> Optional
        // * String -> Text
        // * (anything else) -> Primitive
        match maybe_outer_str
            .unwrap()
            .split(|c| c == ':')
            .last()
            .unwrap()
            .to_lowercase()
            .as_str()
        {
            // If a map, we expect the form to be ...<String, ...> and will
            // verify that the first type paraemter is String
            "hashmap" | "btreemap" => {
                let mut items = inner_str.split(|c| c == ',');
                if let Some(s) = items.next() {
                    if s.trim().to_lowercase().as_str() != "string" {
                        return Err(ParseError::VariantNotFound);
                    }
                }

                let rest = items.collect::<String>();
                Ok(ValueType::Map(Box::from(Self::from_type_name(
                    &rest.trim(),
                )?)))
            }
            "vec" | "vecdeque" | "linkedlist" | "hashset" | "btreeset" | "binaryheap" => Ok(
                ValueType::List(Box::from(Self::from_type_name(&inner_str)?)),
            ),
            "option" => Ok(ValueType::Optional(Box::from(Self::from_type_name(
                &inner_str,
            )?))),
            "string" => Ok(ValueType::Text),
            x => Ok(ValueType::Primitive(PrimitiveValueType::from_type_name(x)?)),
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
            Self::Text => write!(f, "text"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl From<Value> for ValueType {
    fn from(value: Value) -> Self {
        Self::from(&value)
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

impl From<NumberType> for ValueType {
    /// Converts number type (subclass of primitive type) to a value type
    fn from(t: NumberType) -> Self {
        Self::Primitive(PrimitiveValueType::Number(t))
    }
}
