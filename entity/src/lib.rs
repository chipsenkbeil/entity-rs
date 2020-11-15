mod any;
mod database;
mod edge;
mod field;
pub mod query;
mod value;

pub use any::AsAny;
pub use database::{Database, DatabaseError, DatabaseResult};
pub use edge::{Edge, EdgeDefinition, EdgeDefinitionAttribute, EdgeValue, EdgeValueType};
pub use field::{Field, FieldDefinition, FieldDefinitionAttribute};
pub use query::{Condition, Query, QueryExt};
pub use value::{PrimitiveValue, PrimitiveValueType, Value, ValueType};

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

/// Represents the interface for a generic entity whose fields and edges
/// can be accessed by str name regardless of compile-time characteristics
///
/// Based on https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf
pub trait Ent: AsAny {
    /// Represents the unique id associated with each entity instance
    fn id(&self) -> usize;

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    fn r#type(&self) -> &'static str;

    /// Represents the time when the instance of the ent was created
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn created(&self) -> u64;

    /// Represents the time when the instance of the ent was last updated
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn last_updated(&self) -> u64;

    /// Represents fields contained within the ent instance, guaranteeing
    /// that fields are unique by name
    fn fields(&self) -> Vec<&Field>;

    /// Retrieves the field with the provided name within the ent instance
    fn field(&self, name: &str) -> Option<&Field>;

    /// Retrieves the value for the field with the provided name within the
    /// ent instance
    fn field_value(&self, name: &str) -> Option<&Value> {
        self.field(name).map(|f| f.value())
    }

    /// Represents edges between the ent instance and some referred ents
    fn edges(&self) -> Vec<&Edge>;

    /// Retrieves the edge with the provided name within the ent instance
    fn edge(&self, name: &str) -> Option<&Edge>;
}

/// Represents an ent that has no pre-assigned schema and maintains
/// fields and edges using internal maps
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemalessEnt {
    id: usize,
    fields: HashMap<String, Field>,
    edges: HashMap<String, Edge>,
    created: u64,
    last_updated: u64,
}

impl_as_any!(SchemalessEnt);

impl SchemalessEnt {
    /// Creates a new ent using the given id, field map, and edge map
    pub fn new(id: usize, fields: HashMap<String, Field>, edges: HashMap<String, Edge>) -> Self {
        Self {
            id,
            fields,
            edges,
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Invalid system time")
                .as_millis() as u64,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Invalid system time")
                .as_millis() as u64,
        }
    }

    /// Creates an empty map ent with the provided id
    pub fn empty(id: usize) -> Self {
        Self::new(id, HashMap::new(), HashMap::new())
    }

    /// Creates a map ent with the provided id, fields from the given collection,
    /// and edges from the other given collection
    pub fn from_collections(
        id: usize,
        field_collection: impl IntoIterator<Item = Field>,
        edge_collection: impl IntoIterator<Item = Edge>,
    ) -> Self {
        Self::new(
            id,
            field_collection
                .into_iter()
                .map(|f| (f.name().to_string(), f))
                .collect(),
            edge_collection
                .into_iter()
                .map(|e| (e.name().to_string(), e))
                .collect(),
        )
    }
}

impl Default for SchemalessEnt {
    /// Creates an empty map ent using 0 as the id
    fn default() -> Self {
        Self::empty(0)
    }
}

impl Ent for SchemalessEnt {
    /// Represents the unique id associated with each entity instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt};
    ///
    /// let ent = SchemalessEnt::empty(999);
    /// assert_eq!(ent.id(), 999);
    /// ```
    fn id(&self) -> usize {
        self.id
    }

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt};
    ///
    /// let ent = SchemalessEnt::default();
    /// assert_eq!(ent.r#type(), "SchemalessEnt");
    /// ```
    fn r#type(&self) -> &'static str {
        "SchemalessEnt"
    }

    /// Represents the time when the instance of the ent was created
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn created(&self) -> u64 {
        self.created
    }

    /// Represents the time when the instance of the ent was last updated
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn last_updated(&self) -> u64 {
        self.last_updated
    }

    /// Represents fields contained within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt, Field};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = SchemalessEnt::from_collections(0, fields.iter().cloned(), vec![]);
    ///
    /// let ent_fields = ent.fields();
    /// assert_eq!(ent_fields.len(), 2);
    /// assert!(ent_fields.contains(&&Field::new("field1", 123u8)));
    /// assert!(ent_fields.contains(&&Field::new("field2", "some text")));
    /// ```
    fn fields(&self) -> Vec<&Field> {
        self.fields.values().collect()
    }

    /// Retrieves the field with the provided name within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt, Field, Value};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = SchemalessEnt::from_collections(0, fields, vec![]);
    ///
    /// assert_eq!(ent.field("field1").unwrap().value(), &Value::from(123u8));
    /// assert_eq!(ent.field("field???"), None);
    /// ```
    fn field(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }

    /// Represents edges between the ent instance and some referred ents
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt, Edge, EdgeValue as EV};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", EV::OneToOne(99)),
    ///     Edge::new("edge2", EV::OneToMany(vec![1, 2, 3])),
    /// ];
    /// let ent = SchemalessEnt::from_collections(0, vec![], edges);
    ///
    /// let ent_edges = ent.edges();
    /// assert_eq!(ent_edges.len(), 2);
    /// assert!(ent_edges.contains(&&Edge::new("edge1", EV::OneToOne(99))));
    /// assert!(ent_edges.contains(&&Edge::new("edge2", EV::OneToMany(vec![1, 2, 3]))));
    /// ```
    fn edges(&self) -> Vec<&Edge> {
        self.edges.values().collect()
    }

    /// Retrieves the edge with the provided name within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, SchemalessEnt, Edge, EdgeValue as EV};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", EV::OneToOne(99)),
    ///     Edge::new("edge2", EV::OneToMany(vec![1, 2, 3])),
    /// ];
    /// let ent = SchemalessEnt::from_collections(0, vec![], edges);
    ///
    /// assert_eq!(ent.edge("edge1").unwrap().value(), &EV::OneToOne(99));
    /// assert_eq!(ent.edge("edge???"), None);
    /// ```
    fn edge(&self, name: &str) -> Option<&Edge> {
        self.edges.get(name)
    }
}
