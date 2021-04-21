use async_graphql::{
    InputValueError, InputValueResult, Name, Number as AsyncGraphqlNumber, Scalar, ScalarType,
    Value as AsyncGraphqlValue,
};
use derive_more::{From, Into};
use entity::{Number, Primitive, Value, ValueLike};
use std::collections::HashMap;

/// Represents a wrapper around a `Value` to expose as graphql
#[derive(Clone, PartialOrd, PartialEq, Eq, Hash, From, Into)]
pub struct GqlValue(Value);

impl ValueLike for GqlValue {
    fn into_value(self) -> Value {
        self.0
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        Ok(Self(value))
    }
}

#[Scalar]
impl ScalarType for GqlValue {
    fn parse(value: AsyncGraphqlValue) -> InputValueResult<Self> {
        match value {
            AsyncGraphqlValue::Null => Ok(Value::Optional(None)),
            AsyncGraphqlValue::Number(x) => Ok(x
                .as_u64()
                .map(Value::from)
                .or_else(|| x.as_i64().map(Value::from))
                .or_else(|| x.as_f64().map(Value::from))
                .expect("Incoming number not u64/i64/f64")),
            AsyncGraphqlValue::String(x) => Ok(Value::from(x)),
            AsyncGraphqlValue::Boolean(x) => Ok(Value::from(x)),
            AsyncGraphqlValue::List(x) => Ok(Value::from(
                x.into_iter()
                    .map(GqlValue::parse)
                    .collect::<Result<Vec<GqlValue>, InputValueError<Self>>>()?,
            )),
            AsyncGraphqlValue::Object(x) => Ok(Value::from(
                x.into_iter()
                    .map(|(name, value)| {
                        GqlValue::parse(value).map(|value| (name.as_str().to_string(), value))
                    })
                    .collect::<Result<HashMap<String, GqlValue>, InputValueError<Self>>>()?,
            )),
            AsyncGraphqlValue::Enum(_) => Err(InputValueError::expected_type(value)),
        }
        .map(GqlValue::from)
    }

    fn to_value(&self) -> AsyncGraphqlValue {
        match &self.0 {
            Value::List(x) => AsyncGraphqlValue::List(
                x.iter()
                    .map(|x| GqlValue::from(x.clone()).to_value())
                    .collect(),
            ),
            Value::Map(x) => AsyncGraphqlValue::Object(
                x.iter()
                    .map(|(k, v)| (Name::new(k), Self::from(v.clone()).to_value()))
                    .collect(),
            ),
            Value::Optional(None) => AsyncGraphqlValue::Null,
            Value::Optional(Some(x)) => Self::from(*x.clone()).to_value(),
            Value::Primitive(Primitive::Bool(x)) => AsyncGraphqlValue::Boolean(*x),
            Value::Primitive(Primitive::Char(x)) => AsyncGraphqlValue::String(x.to_string()),
            Value::Primitive(Primitive::Unit) => AsyncGraphqlValue::from(()),
            Value::Primitive(Primitive::Number(x)) => match x {
                Number::F32(x) => AsyncGraphqlNumber::from_f64(*x as f64)
                    .map(AsyncGraphqlValue::Number)
                    .unwrap_or(AsyncGraphqlValue::Null),
                Number::F64(x) => AsyncGraphqlNumber::from_f64(*x)
                    .map(AsyncGraphqlValue::Number)
                    .unwrap_or(AsyncGraphqlValue::Null),
                Number::I128(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x as i64)),
                Number::I64(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::I32(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::I16(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::I8(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::Isize(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::U128(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x as u64)),
                Number::U64(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::U32(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::U16(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::U8(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
                Number::Usize(x) => AsyncGraphqlValue::Number(AsyncGraphqlNumber::from(*x)),
            },
            Value::Text(x) => AsyncGraphqlValue::String(x.to_string()),
        }
    }
}
