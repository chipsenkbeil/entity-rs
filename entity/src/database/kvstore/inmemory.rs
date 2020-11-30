use super::{EntIdSet, KeyValueStoreDatabase};
use crate::{
    alloc::{IdAllocator, EPHEMERAL_ID},
    database::{Database, DatabaseError, DatabaseResult},
    ent::{EdgeDeletionPolicy, Ent},
    IEnt, Id,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

/// Represents an in-memory database that performs synchronous insertion,
/// retrieval, and removal. If the feature `serde` is enabled, this database
/// can be serialized and deserialized.
///
/// This database maintains a thread-safe reference-counted mutex around
/// a hashmap representing the global storage. Clones on this database will
/// result in incrementing the reference counter.
///
/// When deserializing the database, any existing reference counter is
/// not persisted, so this can and will duplicate data and fragment users
/// of the database. Best practice is to load the database only when first
/// launching an application!
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InmemoryDatabase {
    /// Primary ent storage
    ents: Arc<Mutex<HashMap<Id, Ent>>>,

    /// Type matching from specific ents to all ids of those ents
    ents_of_type: Arc<Mutex<HashMap<String, EntIdSet>>>,

    /// Id allocator for ents
    alloc: Arc<Mutex<IdAllocator>>,
}

impl Default for InmemoryDatabase {
    /// Creates a new, empty database entry
    fn default() -> Self {
        Self {
            ents: Arc::new(Mutex::new(HashMap::new())),
            ents_of_type: Arc::new(Mutex::new(HashMap::new())),
            alloc: Arc::new(Mutex::new(IdAllocator::new())),
        }
    }
}

impl Database for InmemoryDatabase {
    fn get(&self, id: Id) -> DatabaseResult<Option<Ent>> {
        Ok(self.ents.lock().unwrap().get(&id).map(Clone::clone))
    }

    fn remove(&self, id: Id) -> DatabaseResult<bool> {
        // Remove the ent and, if it has an associated schema, we process
        // each of the edges identified in the schema based on deletion attributes
        if let Some(ent) = self.ents.lock().unwrap().remove(&id) {
            for edge in ent.edges() {
                match edge.deletion_policy() {
                    // If shallow deletion, we only want to remove the connections
                    // back to this ent from the corresponding ents
                    EdgeDeletionPolicy::ShallowDelete => {
                        for edge_id in edge.to_ids() {
                            if let Some(ent) = self.ents.lock().unwrap().get_mut(&edge_id) {
                                let _ = ent.remove_ents_from_all_edges(Some(id));
                            }
                        }
                    }
                    // If deep deletion, we want to remove the ents connected
                    // by the edge
                    EdgeDeletionPolicy::DeepDelete => {
                        for id in edge.to_ids() {
                            let _ = self.remove(id);
                        }
                    }
                    // If deletion policy is nothing, then do nothing
                    EdgeDeletionPolicy::Nothing => {}
                }
            }

            // Remove the id from our type mapping if it is there
            self.ents_of_type
                .lock()
                .unwrap()
                .entry(ent.r#type().to_string())
                .and_modify(|e| {
                    e.remove(&id);
                });

            // Add the id to the freed ids available in the allocator
            self.alloc.lock().unwrap().extend(vec![id]);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn insert(&self, into_ent: impl Into<Ent>) -> DatabaseResult<Id> {
        let mut ent = into_ent.into();

        // Get the id of the ent, swapping out the ephemeral id
        let id = ent.id();
        let id = if id == EPHEMERAL_ID {
            if let Some(id) = self.alloc.lock().unwrap().next() {
                id
            } else {
                return Err(DatabaseError::EntCapacityReached);
            }
        } else {
            self.alloc.lock().unwrap().mark_external_id(id);
            id
        };

        // Update the ent's id to match what is actually to be used
        ent.set_id(id);

        // Add our ent's id to the set of ids associated with the ent's type
        self.ents_of_type
            .lock()
            .unwrap()
            .entry(ent.r#type().to_string())
            .or_insert_with(HashSet::new)
            .insert(id);

        // Add our ent to the primary database
        self.ents.lock().unwrap().insert(id, ent);

        Ok(id)
    }
}

impl KeyValueStoreDatabase for InmemoryDatabase {
    /// Returns ids of all ents stored in the database
    fn ids(&self) -> EntIdSet {
        self.ents.lock().unwrap().keys().copied().collect()
    }

    /// Returns true if database contains the provided id
    fn has_id(&self, id: Id) -> bool {
        self.ents.lock().unwrap().contains_key(&id)
    }

    /// Returns ids of all ents for the given type
    fn ids_for_type(&self, r#type: &str) -> EntIdSet {
        self.ents_of_type
            .lock()
            .unwrap()
            .get(r#type)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Value};

    #[test]
    fn insert_should_replace_ephemeral_id_with_allocator_id() {
        let db = InmemoryDatabase::default();

        let ent = Ent::new_untyped(EPHEMERAL_ID);
        let id = db.insert(ent).expect("Failed to insert ent");
        assert_ne!(id, EPHEMERAL_ID);

        let ent = db.get(id).expect("Failed to get ent").expect("Ent missing");
        assert_eq!(ent.id(), id);
    }

    #[test]
    fn insert_should_add_a_new_ent_using_its_id() {
        let db = InmemoryDatabase::default();

        let ent = Ent::new_untyped(999);
        let id = db.insert(ent).expect("Failed to insert ent");
        assert_eq!(id, 999);

        let ent = db
            .get(999)
            .expect("Failed to get ent")
            .expect("Ent missing");
        assert_eq!(ent.id(), 999);
        assert_eq!(db.alloc.lock().unwrap().next(), Some(1000));
    }

    #[test]
    fn insert_should_overwrite_an_existing_ent_with_the_same_id() {
        let db = InmemoryDatabase::default();

        let ent = Ent::from_collections(
            999,
            Ent::default_type(),
            vec![Field::new("field1", 3)],
            vec![],
        );
        let _ = db.insert(ent).expect("Failed to insert ent");

        let ent = db
            .get(999)
            .expect("Failed to get ent")
            .expect("Ent missing");
        assert_eq!(
            ent.field("field1").expect("Field missing").value(),
            &Value::from(3)
        );
    }

    #[test]
    fn get_should_return_an_ent_by_id() {
        let db = InmemoryDatabase::default();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_none(), "Unexpectedly acquired ent");

        let _ = db.insert(Ent::new_untyped(999)).unwrap();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_some(), "Unexpectedly missing ent");
    }

    #[test]
    fn remove_should_remove_an_ent_by_id() {
        let db = InmemoryDatabase::default();

        let _ = db.remove(999).expect("Failed to remove ent");

        let _ = db.insert(Ent::new_untyped(999)).unwrap();
        assert!(db.get(999).unwrap().is_some(), "Failed to set up ent");

        let _ = db.remove(999).expect("Failed to remove ent");
        assert!(db.get(999).unwrap().is_none(), "Did not remove ent");

        // Id allocator should indicate that id has been freed
        assert_eq!(db.alloc.lock().unwrap().freed(), &[999]);
    }
}
