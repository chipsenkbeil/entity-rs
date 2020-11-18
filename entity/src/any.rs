use std::any::Any;

/// Trait used for casting support into the Any trait object
pub trait AsAny: Any {
    /// Converts reference to Any
    fn as_any(&self) -> &dyn Any;

    /// converts mutable reference to Any
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
