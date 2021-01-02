mod any;
mod edge;
mod field;
pub mod query;
mod value;

pub use any::*;
pub use edge::*;
pub use field::*;
pub use query::*;
pub use value::*;

use crate::{DatabaseError, DatabaseResult, Id, WeakDatabaseRc, EPHEMERAL_ID};
use derive_more::{Display, Error};
use dyn_clone::DynClone;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

/// Represents some error the can occur when mutating an ent
#[derive(Debug, Display, Error)]
pub enum EntMutationError {
    #[display(fmt = "{}", source)]
    BadEdgeValueMutation { source: EdgeValueMutationError },

    #[display(fmt = "Given value for field is wrong type: {}", description)]
    WrongValueType { description: String },

    #[display(fmt = "Given edge value for edge is wrong type: {}", description)]
    WrongEdgeValueType { description: String },

    #[display(fmt = "No edge with name: {}", name)]
    NoEdge { name: String },

    #[display(fmt = "No field with name: {}", name)]
    NoField { name: String },

    #[display(fmt = "Field cannot be updated as it is immutable: {}", name)]
    FieldImmutable { name: String },

    #[display(fmt = "Failed to mark ent as updated: {}", source)]
    MarkUpdatedFailed { source: SystemTimeError },
}

/// Represents some error that can occur when converting an ent to another type
#[derive(Debug, Display, Error)]
pub enum EntConversionError {
    #[display(fmt = "Expected ent of type {}, but got {}", expected, actual)]
    EntWrongType { expected: String, actual: String },
    #[display(fmt = "Missing field {}", name)]
    FieldMissing { name: String },
    #[display(fmt = "Expected field {} to be {}, but was {}", name, expected, actual)]
    FieldWrongType {
        name: String,
        expected: ValueType,
        actual: ValueType,
    },
    #[display(fmt = "Missing edge {}", name)]
    EdgeMissing { name: String },
    #[display(fmt = "Expected edge {} to be {}, but was {}", name, expected, actual)]
    EdgeWrongType {
        name: String,
        expected: EdgeValueType,
        actual: EdgeValueType,
    },
}

/// Represents the interface for an Ent to report its type. This should align
/// with [`Ent::r#type()`] method and is used when we must know the type
/// without having an instance of an ent.
pub trait EntType {
    /// Returns a static str that represents the unique type for an ent
    fn type_str() -> &'static str;
}

/// Represents the interface for a generic entity whose fields and edges
/// can be accessed by str name regardless of compile-time characteristics
///
/// Based on https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf
#[cfg_attr(feature = "serde-1", typetag::serde(tag = "type"))]
pub trait Ent: AsAny + DynClone + Send + Sync {
    /// Represents the unique id associated with each entity instance
    fn id(&self) -> Id;

    /// Updates the id of this ent, useful for databases that want to adjust
    /// the id or when you want to produce a clone of the ent in a database
    /// by resetting its id to ephemeral prior to storing it
    fn set_id(&mut self, id: Id);

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    fn r#type(&self) -> &str;

    /// Represents the time when the instance of the ent was created
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn created(&self) -> u64;

    /// Represents the time when the instance of the ent was last updated
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn last_updated(&self) -> u64;

    /// Updates the time when the instance of the ent was last updated to
    /// the current time in milliseconds since epoch (1970-01-01 00:00:00 UTC)
    fn mark_updated(&mut self) -> Result<(), EntMutationError>;

    /// Returns a list of definitions for fields contained by the ent
    fn field_definitions(&self) -> Vec<FieldDefinition>;

    /// Returns a list of names of fields contained by the ent
    fn field_names(&self) -> Vec<String> {
        self.field_definitions()
            .into_iter()
            .map(|fd| fd.name)
            .collect()
    }

    /// Returns a copy of the value of the field with the specified name
    fn field(&self, name: &str) -> Option<Value>;

    /// Returns a copy of all fields contained by the ent and their associated values
    fn fields(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        for name in self.field_names() {
            if let Some(value) = self.field(&name) {
                fields.push(Field::new(name, value));
            }
        }
        fields
    }

    /// Updates the local value of a field with the specified name, returning
    /// the old field value if updated. This will also update the last updated
    /// time for the ent.
    fn update_field(&mut self, name: &str, value: Value) -> Result<Value, EntMutationError>;

    /// Returns a list of definitions for edges contained by the ent
    fn edge_definitions(&self) -> Vec<EdgeDefinition>;

    /// Returns a list of names of edges contained by the ent
    fn edge_names(&self) -> Vec<String> {
        self.edge_definitions()
            .into_iter()
            .map(|ed| ed.name)
            .collect()
    }

    /// Returns a copy of the value of the edge with the specified name
    fn edge(&self, name: &str) -> Option<EdgeValue>;

    /// Returns a copy of all edges contained by the ent and their associated values
    fn edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        for name in self.edge_names() {
            if let Some(value) = self.edge(&name) {
                edges.push(Edge::new(name, value));
            }
        }
        edges
    }

    /// Updates the local value of an edge with the specified name, returning
    /// the old edge value if updated. This will also update the last updated
    /// time for the ent.
    fn update_edge(&mut self, name: &str, value: EdgeValue) -> Result<EdgeValue, EntMutationError>;

    /// Connects ent to the given database so all future
    /// database-related operations will be performed against this database
    fn connect(&mut self, database: WeakDatabaseRc);

    /// Disconnects ent from any associated database. All future database
    /// operations will fail with a disconnected database error
    fn disconnect(&mut self);

    /// Returns true if ent is currently connected to a database
    fn is_connected(&self) -> bool;

    /// Loads the ents connected by the edge with the given name
    ///
    /// Requires ent to be connected to a database
    fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Box<dyn Ent>>>;

    /// Refreshes ent by checking database for latest version and returning it
    ///
    /// Requires ent to be connected to a database
    fn refresh(&mut self) -> DatabaseResult<()>;

    /// Saves the ent to the database, updating this local instance's id
    /// if the database has reported a new id
    ///
    /// Requires ent to be connected to a database
    fn commit(&mut self) -> DatabaseResult<()>;

    /// Removes self from database, returning true if successful
    ///
    /// Requires ent to be connected to a database
    fn remove(&self) -> DatabaseResult<bool>;
}

/// Blanket implementation for all ents that enables them to be converted
/// to any, which is useful when converting `&dyn Ent` into a concrete type
impl<T: Ent> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

dyn_clone::clone_trait_object!(Ent);

/// Implementation for a generic trait object of [`Ent`] that provides
/// methods to downcast into a concrete type
impl dyn Ent {
    /// Attempts to convert this dynamic Ent ref into a concrete Ent ref
    /// by downcasting
    pub fn as_ent<E: Ent>(&self) -> Option<&E> {
        self.as_any().downcast_ref::<E>()
    }

    /// Attempts to convert this dynamic Ent mutable ref into a concrete Ent
    /// mutable ref by downcasting
    pub fn as_mut_ent<E: Ent>(&mut self) -> Option<&mut E> {
        self.as_mut_any().downcast_mut::<E>()
    }

    /// Attempts to convert this dynamic Ent ref into a concrete ent by
    /// downcasting and then cloning
    pub fn to_ent<E: Ent>(&self) -> Option<E> {
        self.as_ent().map(dyn_clone::clone)
    }
}

pub trait EntExt: Ent {
    /// Loads ents of a specified type from a named edge
    fn load_edge_typed<E: Ent>(&self, name: &str) -> DatabaseResult<Vec<E>>;
}

impl<T: Ent> EntExt for T {
    fn load_edge_typed<E: Ent>(&self, name: &str) -> DatabaseResult<Vec<E>> {
        self.load_edge(name).map(|ents| {
            ents.into_iter()
                .filter_map(|ent| ent.to_ent::<E>())
                .collect()
        })
    }
}

/// Represents a general-purpose ent that is shapeless (no hard type) and
/// maintains fields and edges using internal maps. This ent can optionally
/// be connected to a database and supports additional functionality like
/// loading ents from edges when connected.
#[derive(Clone, Display)]
#[display(fmt = "{} {}", "Self::type_str()", id)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct UntypedEnt {
    #[cfg_attr(feature = "serde-1", serde(skip))]
    database: WeakDatabaseRc,
    id: Id,
    fields: HashMap<String, Field>,
    edges: HashMap<String, Edge>,
    created: u64,
    last_updated: u64,
}

impl fmt::Debug for UntypedEnt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UntypedEnt")
            .field("id", &self.id)
            .field("fields", &self.fields)
            .field("edges", &self.edges)
            .field("created", &self.created)
            .field("last_updated", &self.last_updated)
            .finish()
    }
}

impl Eq for UntypedEnt {}

impl PartialEq for UntypedEnt {
    /// Untyped Ents are considered equal if their ids, fields, edges, creation
    /// date, and updated date are all equal
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.fields == other.fields
            && self.edges == other.edges
            && self.created == other.created
            && self.last_updated == other.last_updated
    }
}

impl UntypedEnt {
    /// Creates a new ent using the given id, field map, and edge map
    pub fn new(id: Id, fields: HashMap<String, Field>, edges: HashMap<String, Edge>) -> Self {
        Self {
            database: WeakDatabaseRc::new(),
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

    /// Creates an empty ent with the provided id
    pub fn empty_with_id(id: Id) -> Self {
        Self::new(id, HashMap::new(), HashMap::new())
    }

    /// Creates a map ent with the provided id, fields from the given
    /// collection, and edges from the other given collection
    pub fn from_collections<FI: IntoIterator<Item = Field>, EI: IntoIterator<Item = Edge>>(
        id: Id,
        field_collection: FI,
        edge_collection: EI,
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

    /// Replaces the ent's local field's value with the given value, returning
    /// the old previous value if the field exists
    ///
    /// If the field does not exist, does NOT insert the value as a new field
    pub fn update_field<N: Into<String>, V: Into<Value>>(
        &mut self,
        into_name: N,
        into_value: V,
    ) -> Result<Value, EntMutationError> {
        self.mark_updated()?;

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
    pub fn add_ents_to_edge<N: Into<String>, I: IntoIterator<Item = Id>>(
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
    pub fn remove_ents_from_edge<N: Into<String>, I: IntoIterator<Item = Id>>(
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
    pub fn remove_ents_from_all_edges<I: IntoIterator<Item = Id>>(
        &mut self,
        ids: I,
    ) -> Result<(), EntMutationError> {
        let edge_ids = ids.into_iter().collect::<HashSet<Id>>();
        let edge_names = self.edges.keys().cloned().collect::<HashSet<String>>();

        for name in edge_names {
            self.remove_ents_from_edge(name, edge_ids.clone())?;
        }

        Ok(())
    }
}

impl Default for UntypedEnt {
    /// Creates an untyped ent with the ephemeral id
    fn default() -> Self {
        Self::empty_with_id(EPHEMERAL_ID)
    }
}

impl EntType for UntypedEnt {
    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{UntypedEnt, EntType};
    ///
    /// assert_eq!(UntypedEnt::type_str(), "entity::ent::UntypedEnt");
    /// ```
    fn type_str() -> &'static str {
        concat!(module_path!(), "::UntypedEnt")
    }
}

#[cfg_attr(feature = "serde-1", typetag::serde)]
impl Ent for UntypedEnt {
    /// Represents the unique id associated with each entity instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt};
    ///
    /// let ent = UntypedEnt::empty_with_id(999);
    /// assert_eq!(ent.id(), 999);
    /// ```
    fn id(&self) -> Id {
        self.id
    }

    /// Updates the id of this ent, useful for databases that want to adjust
    /// the id or when you want to produce a clone of the ent in a database
    /// by resetting its id to ephemeral prior to storing it
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{UntypedEnt, Ent};
    ///
    /// let ent = UntypedEnt::default();
    /// assert_eq!(ent.r#type(), "entity::ent::UntypedEnt");
    /// ```
    fn r#type(&self) -> &str {
        Self::type_str()
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

    /// Updates the local, internal timestamp of this ent instance
    fn mark_updated(&mut self) -> Result<(), EntMutationError> {
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| EntMutationError::MarkUpdatedFailed { source: e })?
            .as_millis() as u64;
        Ok(())
    }

    /// Represents the definitions of fields contained within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Field, FieldDefinition, ValueType};
    /// use std::str::FromStr;
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = UntypedEnt::from_collections(0, fields.iter().cloned(), vec![]);
    ///
    /// let defs = ent.field_definitions();
    /// assert_eq!(defs.len(), 2);
    /// assert!(defs.contains(&FieldDefinition::new(
    ///     "field1",
    ///     ValueType::from_str("u8").unwrap(),
    /// )));
    /// assert!(defs.contains(&FieldDefinition::new(
    ///     "field2",
    ///     ValueType::Text,
    /// )));
    /// ```
    fn field_definitions(&self) -> Vec<FieldDefinition> {
        self.fields.values().map(FieldDefinition::from).collect()
    }

    /// Represents the names of fields contained within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Field};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = UntypedEnt::from_collections(0, fields.iter().cloned(), vec![]);
    ///
    /// let names = ent.field_names();
    /// assert_eq!(names.len(), 2);
    /// assert!(names.contains(&String::from("field1")));
    /// assert!(names.contains(&String::from("field2")));
    /// ```
    fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    /// Returns a copy of the value of the field with the specified name
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Field, Value};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = UntypedEnt::from_collections(0, fields.iter().cloned(), vec![]);
    ///
    /// assert_eq!(ent.field("field1"), Some(Value::from(123u8)));
    /// assert_eq!(ent.field("unknown"), None);
    /// ```
    fn field(&self, name: &str) -> Option<Value> {
        self.fields.get(name).map(|f| f.value().clone())
    }

    /// Updates the local value of a field with the specified name, returning
    /// the old field value if updated
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Field, Value};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let mut ent = UntypedEnt::from_collections(0, fields.iter().cloned(), vec![]);
    ///
    /// ent.update_field("field1", Value::from(5u8)).unwrap();
    /// assert_eq!(ent.field("field1"), Some(Value::from(5u8)));
    /// ```
    fn update_field(&mut self, name: &str, value: Value) -> Result<Value, EntMutationError> {
        self.mark_updated()?;

        match self.fields.entry(name.to_string()) {
            Entry::Occupied(mut x) => {
                let field = Field::new(name.to_string(), value);
                Ok(x.insert(field).into_value())
            }
            Entry::Vacant(_) => Err(EntMutationError::NoField {
                name: name.to_string(),
            }),
        }
    }

    /// Represents the definitions of edges contained within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Edge, EdgeDefinition, EdgeValueType};
    /// use std::str::FromStr;
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", 99),
    ///     Edge::new("edge2", vec![1, 2, 3]),
    /// ];
    /// let ent = UntypedEnt::from_collections(0, vec![], edges);
    ///
    /// let defs = ent.edge_definitions();
    /// assert_eq!(defs.len(), 2);
    /// assert!(defs.contains(&EdgeDefinition::new(
    ///     "edge1",
    ///     EdgeValueType::One,
    /// )));
    /// assert!(defs.contains(&EdgeDefinition::new(
    ///     "edge2",
    ///     EdgeValueType::Many,
    /// )));
    /// ```
    fn edge_definitions(&self) -> Vec<EdgeDefinition> {
        self.edges.values().map(EdgeDefinition::from).collect()
    }

    /// Returns a list of names of edges contained by the ent
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Edge};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", 99),
    ///     Edge::new("edge2", vec![1, 2, 3]),
    /// ];
    /// let ent = UntypedEnt::from_collections(0, vec![], edges);
    ///
    /// let names = ent.edge_names();
    /// assert_eq!(names.len(), 2);
    /// assert!(names.contains(&String::from("edge1")));
    /// assert!(names.contains(&String::from("edge2")));
    /// ```
    fn edge_names(&self) -> Vec<String> {
        self.edges.keys().cloned().collect()
    }

    /// Returns a copy of the value of the edge with the specified name
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Edge, EdgeValue};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", 99),
    ///     Edge::new("edge2", vec![1, 2, 3]),
    /// ];
    /// let ent = UntypedEnt::from_collections(0,  vec![], edges);
    ///
    /// assert_eq!(ent.edge("edge1"), Some(EdgeValue::One(99)));
    /// assert_eq!(ent.edge("edge2"), Some(EdgeValue::Many(vec![1, 2, 3])));
    /// assert_eq!(ent.edge("unknown"), None);
    /// ```
    fn edge(&self, name: &str) -> Option<EdgeValue> {
        self.edges.get(name).map(|e| e.value().clone())
    }

    /// Updates the local value of an edge with the specified name, returning
    /// the old edge value if updated
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, UntypedEnt, Edge, EdgeValue};
    ///
    /// let edges = vec![
    ///     Edge::new("edge1", 99),
    ///     Edge::new("edge2", vec![1, 2, 3]),
    /// ];
    /// let mut ent = UntypedEnt::from_collections(0,  vec![], edges);
    ///
    /// ent.update_edge("edge1", EdgeValue::One(123)).unwrap();
    /// assert_eq!(ent.edge("edge1"), Some(EdgeValue::One(123)));
    /// ```
    fn update_edge(&mut self, name: &str, value: EdgeValue) -> Result<EdgeValue, EntMutationError> {
        self.mark_updated()?;

        match self.edges.entry(name.to_string()) {
            Entry::Occupied(mut x) => {
                let edge = Edge::new(name.to_string(), value);
                Ok(x.insert(edge).into_value())
            }
            Entry::Vacant(_) => Err(EntMutationError::NoEdge {
                name: name.to_string(),
            }),
        }
    }

    /// Connects ent to the given boxed database trait object so all future
    /// database-related operations will be performed against this database
    fn connect(&mut self, database: WeakDatabaseRc) {
        self.database = database;
    }

    /// Disconnects ent from the given database. All future database-related
    /// operations will fail with a disconnected database error
    fn disconnect(&mut self) {
        self.database = WeakDatabaseRc::new();
    }

    /// Returns true if ent is currently connected to a database
    fn is_connected(&self) -> bool {
        WeakDatabaseRc::strong_count(&self.database) > 0
    }

    /// Loads the ents connected by the edge with the given name
    ///
    /// Requires ent to be connected to a database
    fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Box<dyn Ent>>> {
        let database =
            WeakDatabaseRc::upgrade(&self.database).ok_or(DatabaseError::Disconnected)?;
        match self.edge(name) {
            Some(e) => e
                .to_ids()
                .into_iter()
                .filter_map(|id| database.get(id).transpose())
                .collect(),
            None => Err(DatabaseError::MissingEdge {
                name: name.to_string(),
            }),
        }
    }

    /// Refreshes ent by checking database for latest version and returning it
    ///
    /// Requires ent to be connected to a database
    fn refresh(&mut self) -> DatabaseResult<()> {
        let database =
            WeakDatabaseRc::upgrade(&self.database).ok_or(DatabaseError::Disconnected)?;
        let id = self.id;
        match database.get(id)? {
            Some(x) => {
                self.id = x.id();
                self.fields = x
                    .fields()
                    .into_iter()
                    .map(|f| (f.name().to_string(), f.clone()))
                    .collect();
                self.edges = x
                    .edges()
                    .into_iter()
                    .map(|e| (e.name().to_string(), e.clone()))
                    .collect();
                self.created = x.created();
                self.last_updated = x.last_updated();

                Ok(())
            }
            None => Err(DatabaseError::MissingEnt { id }),
        }
    }

    /// Saves the ent to the database, updating this local instance's id
    /// if the database has reported a new id
    ///
    /// Requires ent to be connected to a database
    fn commit(&mut self) -> DatabaseResult<()> {
        let database =
            WeakDatabaseRc::upgrade(&self.database).ok_or(DatabaseError::Disconnected)?;
        match database.insert(Box::new(Self::clone(&self))) {
            Ok(id) => {
                self.set_id(id);
                Ok(())
            }
            Err(x) => Err(x),
        }
    }

    /// Removes self from database, returning true if successful
    ///
    /// Requires ent to be connected to a database
    fn remove(&self) -> DatabaseResult<bool> {
        let database =
            WeakDatabaseRc::upgrade(&self.database).ok_or(DatabaseError::Disconnected)?;
        database.remove(self.id)
    }
}
