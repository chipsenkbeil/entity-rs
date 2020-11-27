mod any;
mod edge;
mod field;
pub mod query;
mod value;

pub use any::AsAny;
pub use edge::{Edge, EdgeDeletionPolicy, EdgeValue, EdgeValueMutationError, EdgeValueType};
pub use field::{Field, FieldAttribute};
pub use query::{
    CollectionCondition, Condition, EdgeCondition, FieldCondition, Query, QueryExt, TimeCondition,
    ValueCondition,
};
pub use value::{
    Number, NumberSign, NumberType, PrimitiveValue, PrimitiveValueType, Value, ValueType,
};

use derive_more::{Display, Error};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

/// Represents some error the can occur when mutating an ent
#[derive(Debug, Display, Error)]
pub enum EntMutationError {
    #[display(fmt = "{}", source)]
    BadEdgeValueMutation { source: EdgeValueMutationError },

    #[display(fmt = "No edge with name: {}", name)]
    NoEdge { name: String },

    #[display(fmt = "No field with name: {}", name)]
    NoField { name: String },
}

/// Represents the interface for a generic entity whose fields and edges
/// can be accessed by str name regardless of compile-time characteristics
///
/// Based on https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf
pub trait IEnt: AsAny {
    /// Represents the unique id associated with each entity instance
    fn id(&self) -> usize;

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    fn r#type(&self) -> &str;

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

/// Blanket implementation for all ents that enables them to be converted
/// to any, which is useful when converting `&dyn Ent` into a concrete type
impl<T: IEnt> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Represents a general-purpose ent that has no pre-assigned type and
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
    /// Returns the Ent struct's default type as a str, which should only
    /// be set for Ent instances that are untyped
    pub const fn default_type() -> &'static str {
        concat!(module_path!(), "::", "Ent")
    }

    /// Creates a new ent using the given id, type, field map, and edge map
    pub fn new(
        id: usize,
        r#type: String,
        fields: HashMap<String, Field>,
        edges: HashMap<String, Edge>,
    ) -> Self {
        Self {
            id,
            r#type,
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

    /// Creates an empty ent with the provided id and type
    pub fn new_empty<T: Into<String>>(id: usize, r#type: T) -> Self {
        Self::new(id, r#type.into(), HashMap::new(), HashMap::new())
    }

    /// Creates an emtpy, untyped ent with the provided id
    pub fn new_untyped(id: usize) -> Self {
        Self::new_empty(id, Self::default_type())
    }

    /// Creates a map ent with the provided id, type, fields from the given
    /// collection, and edges from the other given collection
    pub fn from_collections<
        T: Into<String>,
        FI: IntoIterator<Item = Field>,
        EI: IntoIterator<Item = Edge>,
    >(
        id: usize,
        r#type: T,
        field_collection: FI,
        edge_collection: EI,
    ) -> Self {
        Self::new(
            id,
            r#type.into(),
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

    /// Replaces the ent's local field's value with the given value, returning
    /// the old previous value if the field exists
    ///
    /// If the field does not exist, does NOT insert the value as a new field
    pub fn update_field<N: Into<String>, V: Into<Value>>(
        &mut self,
        into_name: N,
        into_value: V,
    ) -> Result<Value, EntMutationError> {
        self.mark_updated();

        let name = into_name.into();
        let field = Field::new(name.to_string(), into_value);
        match self.fields.entry(name.to_string()) {
            Entry::Occupied(mut x) => Ok(x.insert(field).into_value()),
            Entry::Vacant(_) => Err(EntMutationError::NoField { name }),
        }
    }

    /// Updates the ent's local edge's list to contain the provided ids
    ///
    /// If there are too many ids (in the case of >1 for MaybeOne/One), this
    /// method will fail.
    pub fn add_ents_to_edge<N: Into<String>, I: IntoIterator<Item = usize>>(
        &mut self,
        name: N,
        ids: I,
    ) -> Result<(), EntMutationError> {
        let edge_name = name.into();
        match self.edges.entry(edge_name.to_string()) {
            Entry::Occupied(mut x) => x
                .get_mut()
                .value_mut()
                .add_ids(ids)
                .map_err(|err| EntMutationError::BadEdgeValueMutation { source: err }),
            Entry::Vacant(_) => Err(EntMutationError::NoEdge { name: edge_name }),
        }
    }

    /// Updates the ent's local edge's list to remove the provided ids
    ///
    /// If this would result in an invalid edge (One being empty), this
    /// method will fail.
    pub fn remove_ents_from_edge<N: Into<String>, I: IntoIterator<Item = usize>>(
        &mut self,
        name: N,
        ids: I,
    ) -> Result<(), EntMutationError> {
        let edge_name = name.into();
        match self.edges.entry(edge_name.to_string()) {
            Entry::Occupied(mut x) => x
                .get_mut()
                .value_mut()
                .remove_ids(ids)
                .map_err(|err| EntMutationError::BadEdgeValueMutation { source: err }),
            Entry::Vacant(_) => Err(EntMutationError::NoEdge { name: edge_name }),
        }
    }

    /// Updates all of the ent's local edges to remove the provided ids
    ///
    /// If this would result in an invalid edge (One being empty), this
    /// method will fail.
    pub fn remove_ents_from_all_edges<I: IntoIterator<Item = usize>>(
        &mut self,
        ids: I,
    ) -> Result<(), EntMutationError> {
        let edge_ids = ids.into_iter().collect::<HashSet<usize>>();
        let edge_names = self.edges.keys().cloned().collect::<HashSet<String>>();

        for name in edge_names {
            self.remove_ents_from_edge(name, edge_ids.clone())?;
        }

        Ok(())
    }

    /// Updates the local, internal timestamp of this ent instance
    pub(crate) fn mark_updated(&mut self) {
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Invalid system time")
            .as_millis() as u64;
    }
}

impl Default for Ent {
    /// Creates an untyped ent using 0 as the id and using the default type
    /// for an Ent instance
    fn default() -> Self {
        Self::new_untyped(0)
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
    /// let ent = Ent::new_untyped(999);
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
    fn r#type(&self) -> &str {
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
    ///     Edge::new("edge1", EV::One(99)),
    ///     Edge::new("edge2", EV::Many(vec![1, 2, 3])),
    /// ];
    /// let ent = Ent::from_collections(0, "", vec![], edges);
    ///
    /// let ent_edges = ent.edges();
    /// assert_eq!(ent_edges.len(), 2);
    /// assert!(ent_edges.contains(&&Edge::new("edge1", EV::One(99))));
    /// assert!(ent_edges.contains(&&Edge::new("edge2", EV::Many(vec![1, 2, 3]))));
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
    ///     Edge::new("edge1", EV::One(99)),
    ///     Edge::new("edge2", EV::Many(vec![1, 2, 3])),
    /// ];
    /// let ent = Ent::from_collections(0, "", vec![], edges);
    ///
    /// assert_eq!(ent.edge("edge1").unwrap().value(), &EV::One(99));
    /// assert_eq!(ent.edge("edge???"), None);
    /// ```
    fn edge(&self, name: &str) -> Option<&Edge> {
        self.edges.get(name)
    }
}
