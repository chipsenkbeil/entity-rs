///! Requires async-graphql 2.4.6+
use crate::{
    Edge, EdgeDeletionPolicy, EdgeValue, EdgeValueType, Ent, Field, Id, Number, Predicate,
    PrimitiveValue, Value,
};
use async_graphql::{
    Error, InputObject, InputValueError, InputValueResult, Name, Number as AsyncGraphqlNumber,
    Object, Scalar, ScalarType, Value as AsyncGraphqlValue,
};
use derive_more::{From, Into};
use paste::paste;
use std::collections::HashMap;

#[Object]
impl dyn Ent {
    #[graphql(name = "id")]
    async fn gql_id(&self) -> Id {
        self.id()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> &str {
        self.r#type()
    }

    #[graphql(name = "created")]
    async fn gql_created(&self) -> u64 {
        self.created()
    }

    #[graphql(name = "last_updated")]
    async fn gql_last_updated(&self) -> u64 {
        self.last_updated()
    }

    #[graphql(name = "field")]
    async fn gql_field(&self, name: String) -> Option<Value> {
        self.field(&name)
    }

    #[graphql(name = "fields")]
    async fn gql_fields(&self) -> Vec<Field> {
        self.fields()
    }

    #[graphql(name = "edge")]
    async fn gql_edge(&self, name: String) -> Option<EdgeValue> {
        self.edge(&name)
    }

    #[graphql(name = "edges")]
    async fn gql_edges(&self) -> Vec<Edge> {
        self.edges()
    }

    #[graphql(name = "load_edge")]
    async fn gql_load_edge(&self, name: String) -> async_graphql::Result<Vec<Box<dyn Ent>>> {
        self.load_edge(&name).map_err(|x| Error::new(x.to_string()))
    }
}

#[Object]
impl Field {
    #[graphql(name = "name")]
    async fn gql_name(&self) -> &str {
        self.name()
    }

    #[graphql(name = "value")]
    async fn gql_value(&self) -> &Value {
        self.value()
    }

    #[graphql(name = "indexed")]
    async fn gql_indexed(&self) -> bool {
        self.is_indexed()
    }

    #[graphql(name = "immutable")]
    async fn gql_immutable(&self) -> bool {
        self.is_immutable()
    }
}

#[Object]
impl Edge {
    #[graphql(name = "name")]
    async fn gql_name(&self) -> &str {
        self.name()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> EdgeValueType {
        self.to_type()
    }

    #[graphql(name = "value")]
    async fn gql_value(&self) -> &EdgeValue {
        self.value()
    }

    #[graphql(name = "ids")]
    async fn gql_ids(&self) -> Vec<Id> {
        self.to_ids()
    }

    #[graphql(name = "deletion_policy")]
    async fn gql_deletion_policy(&self) -> EdgeDeletionPolicy {
        self.deletion_policy()
    }
}

#[Object]
impl EdgeValue {
    #[graphql(name = "ids")]
    async fn gql_ids(&self) -> Vec<Id> {
        self.to_ids()
    }

    #[graphql(name = "type")]
    async fn gql_type(&self) -> EdgeValueType {
        self.to_type()
    }
}

// /// Represents a wrapper around an ent query [`Filter`] that exposes a GraphQL API.
// #[derive(InputObject)]
// pub struct GqlFilter {
//     /// Filter by ent's id
//     id: Option<GqlPredicate_Id>,

//     /// Filter by ent's type
//     #[graphql(name = "type")]
//     r#type: Option<GqlPredicate_String>,

//     /// Filter by ent's creation timestamp
//     created: Option<GqlPredicate_u64>,

//     /// Filter by ent's last updated timestamp
//     last_updated: Option<GqlPredicate_u64>,

//     /// Filter by ent's field
//     field: Option<GqlFieldFilter>,

//     /// Filter by ent's edge
//     edge: Option<GqlEdgeFilter>,
// }

// #[derive(InputObject)]
// pub struct GqlFieldFilter {
//     name: String,
//     predicate: GqlPredicate_Value,
// }

// #[derive(InputObject)]
// pub struct GqlEdgeFilter {
//     name: String,
//     filter: Box<GqlFilter>,
// }

// macro_rules! impl_pred {
//     ($type:ty; $($attrs:tt)*) => {
//         paste! {
//             /// Represents a wrapper around an ent query [`TypedPredicate`] that
//             /// exposes a GraphQL API.
//             #[derive(Default, InputObject)]
//             #[allow(non_camel_case_types)]
//             pub struct [<GqlPredicate_ $type>] {
//                 impl_pred!(@field_structs $type; $($attrs)*);
//             }

//             impl From<[<GqlPredicate_ $type>]> for Predicate {
//                 /// Converts into a predicate based on criteria in GraphQL
//                 /// predicate
//                 fn from(x: [<GqlPredicate_ $type>]) -> Self {
//                     let mut criteria = Vec::new();

//                     impl_pred!(@criteria x; criteria; $type; $($attrs)*);

//                     Self::and(criteria)
//                 }
//             }
//         }
//     };
//     (@field_structs $type:ty; @core $($tail:tt)*) => {
//         impl_pred!(@field_struct "Checks if multiple predicates pass", and, Vec<Self>);
//         impl_pred!(@field_struct "Checks if any predicate passes", or, Vec<Self>);
//         impl_pred!(@field_struct "Checks if exactly one predicate passes", xor, Vec<Self>);
//         impl_pred!(@field_struct "Checks if any value in collection passes predicate", any, Box<Self>);
//         impl_pred!(@field_struct "Checks if collection contains value", contains, $type);
//         impl_pred!(@field_struct "Checks if collection contains all values", contains_all, Vec<$type>);
//         impl_pred!(@field_struct "Checks if collection contains any of the values", contains_any, Vec<$type>);
//         impl_pred!(@field_struct "Checks if equals value", equals, $type);
//         impl_pred!(@field_struct "Checks if greater than value", greater_than, $type);
//         impl_pred!(@field_struct "Checks if greater than or equals value", greater_than_or_equals, $type);
//         impl_pred!(@field_struct "Checks if collection contains key", has_key, String);
//         impl_pred!(@field_struct "Checks if collection has key where associated value passes predicate", has_key_where_value, (String, Self));
//         impl_pred!(@field_struct "Checks if value in range", in_range, ($type, $type));
//         impl_pred!(@field_struct "Checks if value in set", in_set, Vec<$type>);
//         impl_pred!(@field_struct "Checks if value is null", is_none, ());
//         impl_pred!(@field_struct "Checks if less than value", less_than, $type);
//         impl_pred!(@field_struct "Checks if less than or equals value", less_than_or_equals, $type);
//         impl_pred!(@field_struct "Checks if does not pass predicate", not, Box<Self>);
//         impl_pred!(@field_struct "Checks if does not equal value", not_equals, $type);
//         impl_pred!(@field_struct "Checks if value not in range", not_in_range, ($type, $type));
//         impl_pred!(@field_struct "Checks if value not in set", not_in_set, Vec<$type>);
//         impl_pred!(@field_structs $type; $($tail)*);
//     };
//     (@field_structs $type:ty; @text $($tail:tt)*) => {
//         impl_pred!(@field_struct "Checks if ends with specified text", text_ends_with, String);
//         impl_pred!(@field_struct "Checks if ends with specified text (case insensitive)", text_ends_with_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if equals specified text (case insensitive)", text_equals_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if text is in set (case insensitive)", text_in_set_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if not equals specified text (case insensitive)", text_not_equals_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if starts with specified text", text_starts_with, String);
//         impl_pred!(@field_struct "Checks if starts with specified text (case insensitive)", text_starts_with_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if text is contained within specified text", text_contained_in, String);
//         impl_pred!(@field_struct "Checks if text is contained within specified text (case insensitive)", text_contained_in_case_insensitive, String);
//         impl_pred!(@field_struct "Checks if text contains all of the specified text within it", text_contains_all, Vec<String>);
//         impl_pred!(@field_struct "Checks if text contains all of the specified text within it (case insensitive)", text_contains_all_case_insensitive, Vec<String>);
//         impl_pred!(@field_struct "Checks if text contains any of the specified text within it", text_contains_any, Vec<String>);
//         impl_pred!(@field_struct "Checks if text contains any of the specified text within it (case insensitive)", text_contains_any_case_insensitive, Vec<String>);
//         impl_pred!(@field_struct "Checks if text ends with any of the specified text", text_ends_with_any, Vec<String>);
//         impl_pred!(@field_struct "Checks if text ends with any of the specified text (case insensitive)", text_ends_with_any_case_insensitive, Vec<String>);
//         impl_pred!(@field_struct "Checks if text starts with any of the specified text", text_starts_with_any, Vec<String>);
//         impl_pred!(@field_struct "Checks if text starts with any of the specified text (case insensitive)", text_starts_with_any_case_insensitive, Vec<String>);
//         impl_pred!(@field_structs $type; $($tail)*);
//     };
//     (@field_structs $type:ty;) => {};
//     (@field_struct $desc:literal, $name:ident, $type:ty) => {
//         #[doc = $desc]
//         $name: Option<$type>,
//     };
//     (@criteria $self:ident; $vec:ident; $type:ty; @core $($tail:tt)*) => {
//         impl_pred!(@criteria_push $self; $vec; and, Vec<Self>, Self::And(v));
//         impl_pred!(@criteria_push $self; $vec; or, Vec<Self>, Self::Or(v));
//         impl_pred!(@criteria_push $self; $vec; xor, Vec<Self>, Self::Xor(v));
//         impl_pred!(@criteria_push $self; $vec; any, Box<Self>, Self::Any(v));
//         impl_pred!(@criteria_push $self; $vec; contains, $type, Self::Contains(v));
//         impl_pred!(@criteria_push $self; $vec; contains_all, Vec<$type>, Self::ContainsAll(v));
//         impl_pred!(@criteria_push $self; $vec; contains_any, Vec<$type>, Self::ContainsAny(v));
//         impl_pred!(@criteria_push $self; $vec; equals, $type, Self::Equals(v));
//         impl_pred!(@criteria_push $self; $vec; greater_than, $type, Self::GreaterThan(v));
//         impl_pred!(@criteria_push $self; $vec; greater_than_or_equals, $type, Self::GreaterThanOrEquals(v));
//         impl_pred!(@criteria_push $self; $vec; has_key, String, Self::HasKey(v));
//         impl_pred!(@criteria_push $self; $vec; has_key_where_value, (String, Self), Self::HasKeyWhereValue(v.0, v.1));
//         impl_pred!(@criteria_push $self; $vec; in_range, ($type, $type), Self::InRange(v.0, v.1));
//         impl_pred!(@criteria_push $self; $vec; in_set, Vec<$type>, Self::InSet(v));
//         impl_pred!(@criteria_push $self; $vec; is_none, (), Self::IsNone);
//         impl_pred!(@criteria_push $self; $vec; less_than, $type, Self::LessThan(v));
//         impl_pred!(@criteria_push $self; $vec; less_than_or_equals, $type, Self::LessThanOrEquals(v));
//         impl_pred!(@criteria_push $self; $vec; not, Box<Self>, Self::Not(v));
//         impl_pred!(@criteria_push $self; $vec; not_equals, $type, Self::NotEquals(v));
//         impl_pred!(@criteria_push $self; $vec; not_in_range, ($type, $type), Self::NotInRange(v.0, v.1));
//         impl_pred!(@criteria_push $self; $vec; not_in_set, Vec<$type>, Self::NotInSet(v));
//         impl_pred!(@criteria $self; $vec; $type; $($tail)*);
//     };
//     (@criteria $self:ident; $vec:ident; $type:ty; @text $($tail:tt)*) => {
//         impl_pred!(@criteria_push $self; $vec; text_ends_with, Self::TextEndsWith(v));
//         impl_pred!(@criteria_push $self; $vec; text_ends_with_case_insensitive, Self::TextEndsWithCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_equals_case_insensitive, Self::TextEqualsCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_in_set_case_insensitive, String, Self::TextInSetCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_not_equals_case_insensitive, String, Self::TextNotEqualsCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_starts_with, String, Self::TextStartsWith(v));
//         impl_pred!(@criteria_push $self; $vec; text_starts_with_case_insensitive, String, Self::TextStartsWithCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_contained_in, String, Self::TextContainedIn(v));
//         impl_pred!(@criteria_push $self; $vec; text_contained_in_case_insensitive, String, Self::TextContainedInCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_contains_all, Vec<String>, Self::TextContainsAll(v));
//         impl_pred!(@criteria_push $self; $vec; text_contains_all_case_insensitive, Vec<String>, Self::TextContainsAllCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_contains_any, Vec<String>, Self::TextContainsAny(v));
//         impl_pred!(@criteria_push $self; $vec; text_contains_any_case_insensitive, Vec<String>, Self::TextContainsAnyCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_ends_with_any, Vec<String>, Self::TextEndsWithAny(v));
//         impl_pred!(@criteria_push $self; $vec; text_ends_with_any_case_insensitive, Vec<String>, Self::TextEndsWithAnyCaseInsensitive(v));
//         impl_pred!(@criteria_push $self; $vec; text_starts_with_any, Vec<String>, Self::TextStartsWithAny(v));
//         impl_pred!(@criteria_push $self; $vec; text_starts_with_any_case_insensitive, Vec<String>, Self::TextStartsWithAnyCaseInsensitive(v));
//         impl_pred!(@criteria $self; $vec; $type; $($tail)*);
//     };
//     (@criteria $self:ident; $vec:ident; $type:ty;) => {};
//     (@criteria_push $self:ident; $vec:ident; $name:ident, $make_pred:expr) => {
//         if let Some(v) = $self.$name {
//             let p = $make_pred;
//             $vec.push(p);
//         }
//     };
// }

// impl_pred!(Value; @core @text);
// impl_pred!(String; @core @text);
// impl_pred!(Id; @core);
// impl_pred!(u64; @core);

#[Scalar]
impl ScalarType for Value {
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
                    .map(Value::parse)
                    .collect::<Result<Vec<Value>, InputValueError<Self>>>()?,
            )),
            AsyncGraphqlValue::Object(x) => Ok(Value::from(
                x.into_iter()
                    .map(|(name, value)| {
                        Value::parse(value).map(|value| (name.as_str().to_string(), value))
                    })
                    .collect::<Result<HashMap<String, Value>, InputValueError<Self>>>()?,
            )),
            AsyncGraphqlValue::Enum(_) => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> AsyncGraphqlValue {
        match self {
            Self::List(x) => AsyncGraphqlValue::List(x.iter().map(ScalarType::to_value).collect()),
            Self::Map(x) => AsyncGraphqlValue::Object(
                x.iter()
                    .map(|(k, v)| (Name::new(k), v.to_value()))
                    .collect(),
            ),
            Self::Optional(None) => AsyncGraphqlValue::Null,
            Self::Optional(Some(x)) => x.to_value(),
            Self::Primitive(PrimitiveValue::Bool(x)) => AsyncGraphqlValue::Boolean(*x),
            Self::Primitive(PrimitiveValue::Char(x)) => AsyncGraphqlValue::String(x.to_string()),
            Self::Primitive(PrimitiveValue::Unit) => AsyncGraphqlValue::from(()),
            Self::Primitive(PrimitiveValue::Number(x)) => match x {
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
            Self::Text(x) => AsyncGraphqlValue::String(x.to_string()),
        }
    }
}
