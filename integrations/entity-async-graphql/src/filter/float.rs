use async_graphql::InputObject;
use entity::Predicate;
use paste::paste;

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
                #[doc = "Checks if value is null"] is_none: Option<bool>,
                #[doc = "Checks if less than value"] less_than: Option<$type>,
                #[doc = "Checks if less than or equals value"] less_than_or_equals: Option<$type>,
                #[doc = "Checks if does not pass predicate"] not: Option<Box<Self>>,
                #[doc = "Checks if does not equal value"] not_equals: Option<$type>,
                #[doc = "Checks if value not in range"] not_in_range: Option<[<GqlPredicate_ $name _RangeArgs>]>,
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
            impl_pred!(@criteria $self; $vec; $name; $type; $($tail)*);
        }
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

impl_pred!(f32; f32; @core);
impl_pred!(f64; f64; @core);
