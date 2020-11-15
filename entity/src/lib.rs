mod any;
mod context;
mod field;
pub mod query;
mod value;

pub use any::AsAny;
pub use context::Context;
pub use field::{Field, FieldDefinition};
pub use query::{Query, QueryExt};
pub use value::{PrimitiveValue, PrimitiveValueType, Value, ValueType};

use derive_more::Constructor;
use std::collections::HashMap;

/// Represents a generic entity
pub trait Ent: AsAny {
    /// Represents the unique id associated with each entity instance
    fn id(&self) -> usize;

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    fn r#type(&self) -> &'static str;

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
}

/// Represents an ent that uses an internal map to contain fields dynamically
#[derive(Constructor, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapEnt(usize, HashMap<String, Field>);

impl_as_any!(MapEnt);

impl MapEnt {
    /// Creates an empty map ent with the provided id
    pub fn empty(id: usize) -> Self {
        Self::new(id, HashMap::new())
    }
}

impl Default for MapEnt {
    /// Creates an empty map ent using 0 as the id
    fn default() -> Self {
        Self::empty(0)
    }
}

impl Ent for MapEnt {
    /// Represents the unique id associated with each entity instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, MapEnt};
    ///
    /// let ent = MapEnt::empty(999);
    /// assert_eq!(ent.id(), 999);
    /// ```
    fn id(&self) -> usize {
        self.0
    }

    /// Represents a unique type associated with the entity, used for
    /// lookups, indexing by type, and conversions
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, MapEnt};
    ///
    /// let ent = MapEnt::default();
    /// assert_eq!(ent.r#type(), "MapEnt");
    /// ```
    fn r#type(&self) -> &'static str {
        "MapEnt"
    }

    /// Represents fields contained within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, MapEnt, Field};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = MapEnt::new(0, fields.iter().map(|f| (f.name().to_string(), f.clone())).collect());
    ///
    /// let ent_fields = ent.fields();
    /// assert_eq!(ent_fields.len(), 2);
    /// assert!(ent_fields.contains(&&Field::new("field1", 123u8)));
    /// assert!(ent_fields.contains(&&Field::new("field2", "some text")));
    /// ```
    fn fields(&self) -> Vec<&Field> {
        self.1.values().collect()
    }

    /// Retrieves the field with the provided name within the ent instance
    ///
    /// ## Examples
    ///
    /// ```
    /// use entity::{Ent, MapEnt, Field, Value};
    ///
    /// let fields = vec![
    ///     Field::new("field1", 123u8),
    ///     Field::new("field2", "some text"),
    /// ];
    /// let ent = MapEnt::new(0, fields.iter().map(|f| (f.name().to_string(), f.clone())).collect());
    ///
    /// assert_eq!(ent.field("field1").unwrap().value(), &Value::from(123u8));
    /// assert_eq!(ent.field("field???"), None);
    /// ```
    fn field(&self, name: &str) -> Option<&Field> {
        self.1.get(name)
    }
}
