use std::any::Any;

/// Trait used for casting support into the Any trait object
pub trait AsAny: Any {
    /// Converts reference to Any
    fn as_any(&self) -> &dyn Any;

    /// converts mutable reference to Any
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

/// Generates implementation of AsAny trait for the given identifier and
/// optional set of lifetimes and generic types
///
/// ## Examples
///
/// ```
/// use entity::{AsAny, impl_as_any};
///
/// pub struct SomeType;
/// impl_as_any!(SomeType);
///
/// pub struct SomeGenericType<T: 'static> {
///     inner: T,
/// }
/// impl_as_any!(SomeGenericType, T);
/// ```
#[macro_export]
macro_rules! impl_as_any {
    ($name:ident, $($generic:tt),+) => {
        impl<$($generic),+> AsAny for $name<$($generic),+> {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
    ($name:ident) => {
        impl AsAny for $name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}
