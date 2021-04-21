use crate::GqlValue;
use async_graphql::InputObject;
use entity::{Id, Predicate};
use paste::paste;
use std::collections::HashSet;

macro_rules! impl_pred {
    ($name:ident; $type:ty; $($attrs:tt)*) => {
        paste! {
            #[derive(Clone, InputObject)]
            #[graphql(rename_fields = "snake_case")]
            #[allow(non_camel_case_types)]
            pub struct [<GqlPredicate_ $name _RangeArgs>] {
                start: $type,
                end: $type,
            }

            #[derive(Clone, InputObject)]
            #[graphql(rename_fields = "snake_case")]
            #[allow(non_camel_case_types)]
            pub struct [<GqlPredicate_ $name _HasKeyWhereValueArgs>] {
                key: String,
                predicate: Box<[<GqlPredicate_ $name>]>,
            }

            /// Represents a wrapper around an ent query [`TypedPredicate`] that
            /// exposes a GraphQL API.
            #[derive(Clone, Default, InputObject)]
            #[graphql(rename_fields = "snake_case")]
            #[allow(non_camel_case_types)]
            pub struct [<GqlPredicate_ $name>] {
                #[doc = "Checks if multiple predicates pass"] and: Option<Vec<Self>>,
                #[doc = "Checks if any predicate passes"] or: Option<Vec<Self>>,
                #[doc = "Checks if exactly one predicate passes"] xor: Option<Vec<Self>>,
                #[doc = "Checks if any value in collection passes predicate"] any: Option<Box<Self>>,
                #[doc = "Checks if collection contains value"] contains: Option<$type>,
                #[doc = "Checks if collection contains all values"] contains_all: Option<Vec<$type>>,
                #[doc = "Checks if collection contains any of the values"] contains_any: Option<Vec<$type>>,
                #[doc = "Checks if equals value"] equals: Option<$type>,
                #[doc = "Checks if greater than value"] greater_than: Option<$type>,
                #[doc = "Checks if greater than or equals value"] greater_than_or_equals: Option<$type>,
                #[doc = "Checks if collection contains key"] has_key: Option<String>,
                #[doc = "Checks if collection has key where associated value passes predicate"] has_key_where_value: Option<[<GqlPredicate_ $name _HasKeyWhereValueArgs>]>,
                #[doc = "Checks if value in range"] in_range: Option<[<GqlPredicate_ $name _RangeArgs>]>,
                #[doc = "Checks if value in set"] in_set: Option<HashSet<$type>>,
                #[doc = "Checks if value is null"] is_none: Option<bool>,
                #[doc = "Checks if less than value"] less_than: Option<$type>,
                #[doc = "Checks if less than or equals value"] less_than_or_equals: Option<$type>,
                #[doc = "Checks if does not pass predicate"] not: Option<Box<Self>>,
                #[doc = "Checks if does not equal value"] not_equals: Option<$type>,
                #[doc = "Checks if value not in range"] not_in_range: Option<[<GqlPredicate_ $name _RangeArgs>]>,
                #[doc = "Checks if value not in set"] not_in_set: Option<HashSet<$type>>,
                #[doc = "Checks if ends with specified text"] text_ends_with: Option<String>,
                #[doc = "Checks if ends with specified text (case insensitive)"] text_ends_with_case_insensitive: Option<String>,
                #[doc = "Checks if equals specified text (case insensitive)"] text_equals_case_insensitive: Option<String>,
                #[doc = "Checks if text is in set (case insensitive)"] text_in_set_case_insensitive: Option<HashSet<String>>,
                #[doc = "Checks if not equals specified text (case insensitive)"] text_not_equals_case_insensitive: Option<String>,
                #[doc = "Checks if starts with specified text"] text_starts_with: Option<String>,
                #[doc = "Checks if starts with specified text (case insensitive)"] text_starts_with_case_insensitive: Option<String>,
                #[doc = "Checks if text is contained within specified text"] text_contained_in: Option<String>,
                #[doc = "Checks if text is contained within specified text (case insensitive)"] text_contained_in_case_insensitive: Option<String>,
                #[doc = "Checks if text contains all of the specified text within it"] text_contains_all: Option<Vec<String>>,
                #[doc = "Checks if text contains all of the specified text within it (case insensitive)"] text_contains_all_case_insensitive: Option<Vec<String>>,
                #[doc = "Checks if text contains any of the specified text within it"] text_contains_any: Option<Vec<String>>,
                #[doc = "Checks if text contains any of the specified text within it (case insensitive)"] text_contains_any_case_insensitive: Option<Vec<String>>,
                #[doc = "Checks if text ends with any of the specified text"] text_ends_with_any: Option<Vec<String>>,
                #[doc = "Checks if text ends with any of the specified text (case insensitive)"] text_ends_with_any_case_insensitive: Option<Vec<String>>,
                #[doc = "Checks if text starts with any of the specified text"] text_starts_with_any: Option<Vec<String>>,
                #[doc = "Checks if text starts with any of the specified text (case insensitive)"] text_starts_with_any_case_insensitive: Option<Vec<String>>,
            }

            impl From<Box<[<GqlPredicate_ $name>]>> for Predicate {
                fn from(x: Box<[<GqlPredicate_ $name>]>) -> Self {
                    Self::from(x.as_ref().clone())
                }
            }

            impl From<[<GqlPredicate_ $name>]> for Predicate {
                /// Converts into a predicate based on criteria in GraphQL
                /// predicate
                fn from(x: [<GqlPredicate_ $name>]) -> Self {
                    let mut criteria = Vec::new();

                    impl_pred!(@criteria x; criteria; $name; $type; $($attrs)*);

                    Self::and(criteria)
                }
            }
        }
    };
    (@criteria $self:ident; $vec:ident; $name:ident; $type:ty; @core $($tail:tt)*) => {
        paste! {
            impl_pred!(@criteria_push $self; $vec; and; |v| Self::and(v));
            impl_pred!(@criteria_push $self; $vec; or; |v| Self::or(v));
            impl_pred!(@criteria_push $self; $vec; xor; |v| Self::xor(v));
            impl_pred!(
                @criteria_push $self; $vec; any;
                |v: Box<[<GqlPredicate_ $name>]>| Self::any(v.as_ref().clone())
            );
            impl_pred!(@criteria_push $self; $vec; contains; |v| Self::contains(v));
            impl_pred!(@criteria_push $self; $vec; contains_all; |v| Self::contains_all(v));
            impl_pred!(@criteria_push $self; $vec; contains_any; |v| Self::contains_any(v));
            impl_pred!(@criteria_push $self; $vec; equals; |v| Self::equals(v));
            impl_pred!(@criteria_push $self; $vec; greater_than; |v| Self::greater_than(v));
            impl_pred!(@criteria_push $self; $vec; greater_than_or_equals; |v| Self::greater_than_or_equals(v));
            impl_pred!(@criteria_push $self; $vec; has_key; |v| Self::has_key(v));
            impl_pred!(
                @criteria_push $self; $vec; has_key_where_value;
                |v: [<GqlPredicate_ $name _HasKeyWhereValueArgs>]|
                Self::has_key_where_value(v.key, v.predicate.as_ref().clone())
            );
            impl_pred!(
                @criteria_push $self; $vec; in_range;
                |v: [<GqlPredicate_ $name _RangeArgs>]| Self::in_range(v.start..=v.end)
            );
            impl_pred!(@criteria_push $self; $vec; in_set; |v| Self::in_set(v));
            impl_pred!(@criteria_push $self; $vec; is_none; |v| if v { Self::IsNone } else { Self::not(Self::IsNone) });
            impl_pred!(@criteria_push $self; $vec; less_than; |v| Self::less_than(v));
            impl_pred!(@criteria_push $self; $vec; less_than_or_equals; |v| Self::less_than_or_equals(v));
            impl_pred!(
                @criteria_push $self; $vec; not;
                |v: Box<[<GqlPredicate_ $name>]>| Self::not(v.as_ref().clone())
            );
            impl_pred!(@criteria_push $self; $vec; not_equals; |v| Self::not_equals(v));
            impl_pred!(
                @criteria_push $self; $vec; not_in_range;
                |v: [<GqlPredicate_ $name _RangeArgs>]| Self::not_in_range(v.start..=v.end)
            );
            impl_pred!(@criteria_push $self; $vec; not_in_set; |v| Self::not_in_set(v));
            impl_pred!(@criteria $self; $vec; $name; $type; $($tail)*);
        }
    };
    (@criteria $self:ident; $vec:ident; $name:ident; $type:ty; @text $($tail:tt)*) => {
        impl_pred!(@criteria_push $self; $vec; text_ends_with; |v| Self::TextEndsWith(v));
        impl_pred!(@criteria_push $self; $vec; text_ends_with_case_insensitive; |v| Self::TextEndsWithCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_equals_case_insensitive; |v| Self::TextEqualsCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_in_set_case_insensitive; |v| Self::TextInSetCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_not_equals_case_insensitive; |v| Self::TextNotEqualsCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_starts_with; |v| Self::TextStartsWith(v));
        impl_pred!(@criteria_push $self; $vec; text_starts_with_case_insensitive; |v| Self::TextStartsWithCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_contained_in; |v| Self::TextContainedIn(v));
        impl_pred!(@criteria_push $self; $vec; text_contained_in_case_insensitive; |v| Self::TextContainedInCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_contains_all; |v| Self::TextContainsAll(v));
        impl_pred!(@criteria_push $self; $vec; text_contains_all_case_insensitive; |v| Self::TextContainsAllCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_contains_any; |v| Self::TextContainsAny(v));
        impl_pred!(@criteria_push $self; $vec; text_contains_any_case_insensitive; |v| Self::TextContainsAnyCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_ends_with_any; |v| Self::TextEndsWithAny(v));
        impl_pred!(@criteria_push $self; $vec; text_ends_with_any_case_insensitive; |v| Self::TextEndsWithAnyCaseInsensitive(v));
        impl_pred!(@criteria_push $self; $vec; text_starts_with_any; |v| Self::TextStartsWithAny(v));
        impl_pred!(@criteria_push $self; $vec; text_starts_with_any_case_insensitive; |v| Self::TextStartsWithAnyCaseInsensitive(v));
        impl_pred!(@criteria $self; $vec; $name; $type; $($tail)*);
    };
    (@criteria $self:ident; $vec:ident; $name:ident; $type:ty;) => {};
    (@criteria_push $self:ident; $vec:ident; $name:ident; $make_pred:expr) => {
        if let Some(v) = $self.$name {
            let f = $make_pred;
            let p = f(v);
            $vec.push(p);
        }
    };
}

impl_pred!(Value; GqlValue; @core @text);
impl_pred!(String; String; @core @text);

impl_pred!(Id; Id; @core);
impl_pred!(bool; bool; @core);
impl_pred!(char; char; @core);

// NOTE: async-graphql does not support i128
impl_pred!(isize; isize; @core);
impl_pred!(i64; i64; @core);
impl_pred!(i32; i32; @core);
impl_pred!(i16; i16; @core);
impl_pred!(i8; i8; @core);

// NOTE: async-graphql does not support u128
impl_pred!(usize; usize; @core);
impl_pred!(u64; u64; @core);
impl_pred!(u32; u32; @core);
impl_pred!(u16; u16; @core);
impl_pred!(u8; u8; @core);
