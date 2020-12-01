mod any;
mod edge;
mod field;
mod r#macro;
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

use crate::{Database, DatabaseError, DatabaseResult, Id};
use derive_more::{Display, Error};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
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
    fn id(&self) -> Id;

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
/// maintains fields and edges using internal maps. This ent can optionally
/// be connected to a database and supports additional functionality like
/// loading ents from edges when connected.
#[derive(Clone, Display)]
#[display(fmt = "Ent {} of type {}", id, r#type)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ent {
    #[cfg_attr(feature = "serde", serde(skip))]
    database: Option<Box<dyn Database>>,
    id: Id,
    r#type: String,
    fields: HashMap<String, Field>,
    edges: HashMap<String, Edge>,
    created: u64,
    last_updated: u64,
}

impl fmt::Debug for Ent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ent")
            .field("id", &self.id)
            .field("r#type", &self.r#type)
            .field("fields", &self.fields)
            .field("edges", &self.edges)
            .field("created", &self.created)
            .field("last_updated", &self.last_updated)
            .finish()
    }
}

impl Eq for Ent {}

impl PartialEq for Ent {
    /// Ents are considered equal if their ids, types, fields, edges, creation
    /// date, and updated date are all equal
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.r#type == other.r#type
            && self.fields == other.fields
            && self.edges == other.edges
            && self.created == other.created
            && self.last_updated == other.last_updated
    }
}

impl Ent {
    /// Connects ent to the given database so all future database-related
    /// operations will be performed against this database
    #[inline]
    pub fn connect<D: 'static + Database>(&mut self, database: D) -> &mut Self {
        self.connect_boxed(Box::from(database))
    }

    /// Connects ent to the given boxed database trait object so all future
    /// database-related operations will be performed against this database
    pub fn connect_boxed(&mut self, database: Box<dyn Database>) -> &mut Self {
        self.database = Some(database);
        self
    }

    /// Disconnects ent from the given database. All future database-related
    /// operations will fail with a disconnected database error
    pub fn disconnect(&mut self) -> &mut Self {
        self.database = None;
        self
    }

    /// Returns true if ent is currently connected to a database
    pub fn is_connected(&self) -> bool {
        self.database.is_some()
    }

    /// Loads the ents connected by the edge with the given name
    ///
    /// Requires ent to be connected to a database
    pub fn load_edge(&self, name: &str) -> DatabaseResult<Vec<Ent>> {
        let database = self.database.as_ref().ok_or(DatabaseError::Disconnected)?;
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
    pub fn refresh(&mut self) -> DatabaseResult<()> {
        let database = self.database.as_ref().ok_or(DatabaseError::Disconnected)?;
        let id = self.id;
        match database.get(id)? {
            Some(x) => {
                self.id = x.id;
                self.r#type = x.r#type;
                self.fields = x.fields;
                self.edges = x.edges;
                self.created = x.created;
                self.last_updated = x.last_updated;

                Ok(())
            }
            None => Err(DatabaseError::MissingEnt { id }),
        }
    }

    /// Saves the ent to the database, updating this local instance's id
    /// if the database has reported a new id
    ///
    /// Requires ent to be connected to a database
    pub fn commit(&mut self) -> DatabaseResult<()> {
        let database = self.database.as_ref().ok_or(DatabaseError::Disconnected)?;
        match database.insert(self.clone()) {
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
    pub fn remove(self) -> DatabaseResult<bool> {
        let database = self.database.as_ref().ok_or(DatabaseError::Disconnected)?;
        database.remove(self.id)
    }
}

impl Ent {
    /// Returns the Ent struct's default type as a str, which should only
    /// be set for Ent instances that are untyped
    pub const fn default_type() -> &'static str {
        concat!(module_path!(), "::", "Ent")
    }

    /// Creates a new ent using the given id, type, field map, and edge map
    pub fn new(
        id: Id,
        r#type: String,
        fields: HashMap<String, Field>,
        edges: HashMap<String, Edge>,
    ) -> Self {
        Self {
            database: None,
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
    pub fn new_empty<T: Into<String>>(id: Id, r#type: T) -> Self {
        Self::new(id, r#type.into(), HashMap::new(), HashMap::new())
    }

    /// Creates an emtpy, untyped ent with the provided id
    pub fn new_untyped(id: Id) -> Self {
        Self::new_empty(id, Self::default_type())
    }

    /// Creates a map ent with the provided id, type, fields from the given
    /// collection, and edges from the other given collection
    pub fn from_collections<
        T: Into<String>,
        FI: IntoIterator<Item = Field>,
        EI: IntoIterator<Item = Edge>,
    >(
        id: Id,
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

    /// Updates the id of this ent, useful for databases that want to adjust
    /// the id or when you want to produce a clone of the ent in a database
    /// by resetting its id to ephemeral prior to storing it
    pub fn set_id(&mut self, id: Id) {
        self.id = id;
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
    fn id(&self) -> Id {
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
