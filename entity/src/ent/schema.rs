use super::{edge::EdgeDefinition, field::FieldDefinition};

/// Represents the schema associated with some concrete ent type. It is
/// expected that each concrete schema type is defined at compile-time and
/// is consistent when accessing its field and edge definitions
pub trait IEntSchema {
    /// Returns the fields expected for the concrete ent
    fn fields(&self) -> &[FieldDefinition];

    /// Returns the edges expected for the concrete ent
    fn edges(&self) -> &[EdgeDefinition];
}

/// Represents a schema that can be defined once with arbitrary definitions
#[derive(Clone, Debug)]
pub struct EntSchema {
    field_defs: Vec<FieldDefinition>,
    edge_defs: Vec<EdgeDefinition>,
}

impl EntSchema {
    pub fn new(
        into_field_defs: impl IntoIterator<Item = FieldDefinition>,
        into_edge_defs: impl IntoIterator<Item = EdgeDefinition>,
    ) -> Self {
        Self {
            field_defs: into_field_defs.into_iter().collect(),
            edge_defs: into_edge_defs.into_iter().collect(),
        }
    }
}

impl IEntSchema for EntSchema {
    fn fields(&self) -> &[FieldDefinition] {
        &self.field_defs
    }

    fn edges(&self) -> &[EdgeDefinition] {
        &self.edge_defs
    }
}
