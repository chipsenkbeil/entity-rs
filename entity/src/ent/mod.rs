mod edge;
mod field;
pub mod query;
mod schema;
mod value;

pub use edge::{Edge, EdgeDefinition, EdgeDefinitionAttribute, EdgeValue, EdgeValueType};
pub use field::{Field, FieldDefinition, FieldDefinitionAttribute};
pub use query::{Condition, FieldCondition, Query, QueryExt};
pub use schema::{EmptyEntSchema, EntSchema};
pub use value::{PrimitiveValue, PrimitiveValueType, Value, ValueType};

use super::IEnt;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

/// Represents a general-purpose ent that has no pre-assigned schema and
/// maintains fields and edges using internal maps
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ent {
    id: usize,
    r#type: String,
    fields: HashMap<String, Field>,
    edges: HashMap<String, Edge>,
    created: u64,
    last_updated: u64,
}

impl Ent {
    /// Creates a new ent using the given id, type, field map, and edge map
    pub fn new(
        id: usize,
        r#type: impl Into<String>,
        fields: HashMap<String, Field>,
        edges: HashMap<String, Edge>,
    ) -> Self {
        Self {
            id,
            r#type: r#type.into(),
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

    /// Creates an empty map ent with the provided id and type
    pub fn empty(id: usize, r#type: impl Into<String>) -> Self {
        Self::new(id, r#type, HashMap::new(), HashMap::new())
    }

    /// Creates a map ent with the provided id, type, fields from the given
    /// collection, and edges from the other given collection
    pub fn from_collections(
        id: usize,
        r#type: impl Into<String>,
        field_collection: impl IntoIterator<Item = Field>,
        edge_collection: impl IntoIterator<Item = Edge>,
    ) -> Self {
        Self::new(
            id,
            r#type,
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

impl Default for Ent {
    /// Creates an empty map ent using 0 as the id and providing a default
    /// type string
    fn default() -> Self {
        Self::empty(0, concat!(module_path!(), "::", "Ent"))
    }
}

impl IEnt for Ent {
    /// Represents the unique id associated with each entity instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, IEnt};
    ///
    /// let ent = Ent::empty(999);
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
    /// use entity::{Ent, IEnt};
    ///
    /// let ent = Ent::default();
    /// assert_eq!(ent.r#type(), "entity::ent::Ent");
    /// ```
    fn r#type<'a>(&'a self) -> &'a str {
        &self.r#type
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
    /// use entity::{Ent, IEnt, Field};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = Ent::from_collections(0, "", fields.iter().cloned(), vec![]);
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
    /// use entity::{Ent, IEnt, Field, Value};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = Ent::from_collections(0, "", fields, vec![]);
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
    /// use entity::{Ent, IEnt, Edge, EdgeValue as EV};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", EV::OneToOne(99)),
    ///     Edge::new("edge2", EV::OneToMany(vec![1, 2, 3])),
    /// ];
    /// let ent = Ent::from_collections(0, "", vec![], edges);
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
    /// use entity::{Ent, IEnt, Edge, EdgeValue as EV};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", EV::OneToOne(99)),
    ///     Edge::new("edge2", EV::OneToMany(vec![1, 2, 3])),
    /// ];
    /// let ent = Ent::from_collections(0, "", vec![], edges);
    ///
    /// assert_eq!(ent.edge("edge1").unwrap().value(), &EV::OneToOne(99));
    /// assert_eq!(ent.edge("edge???"), None);
    /// ```
    fn edge(&self, name: &str) -> Option<&Edge> {
        self.edges.get(name)
    }
}
