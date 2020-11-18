use super::{edge::EdgeDefinition, field::FieldDefinition};

/// Represents the schema associated with some concrete ent type. It is
/// expected that each concrete schema type is defined at compile-time and
/// is consistent when accessing its field and edge definitions
pub trait EntSchema {
    /// Static method that returns the fields expected for the concrete ent
    fn fields() -> Vec<FieldDefinition>;

    /// Static method that returns the edges expected for the concrete ent
    fn edges() -> Vec<EdgeDefinition>;
}

/// Represents an empty ent schema, meaning that it has no fields or edges
/// according to the schema itself
pub struct EmptyEntSchema;

impl EntSchema for EmptyEntSchema {
    fn fields() -> Vec<FieldDefinition> {
        vec![]
    }

    fn edges() -> Vec<EdgeDefinition> {
        vec![]
    }
}
