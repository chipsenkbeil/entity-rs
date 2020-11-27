use super::{EntIdSet, KeyValueStoreDatabase};
use crate::{
    database::{Database, DatabaseError, DatabaseResult},
    ent::{EdgeDeletionPolicy, Ent},
    IEnt,
};
use derive_more::Constructor;
use std::collections::HashSet;

/// Represents a sled database that performs synchronous insertion,
/// retrieval, and removal. Sled maintains disk-backed data, so the `serde`
/// feature has no purpose with this database.
///
/// Sled itself is thread-safe, maintaining an internal `Arc` for each tree;
/// therefore, this database can be cloned to increment those counters.
#[derive(Constructor, Clone)]
pub struct SledDatabase(sled::Db);

fn id_to_ivec(id: usize) -> sled::IVec {
    id.to_be_bytes().as_ref().into()
}

fn ivec_to_id(ivec: sled::IVec) -> Option<usize> {
    use std::convert::TryInto;
    let (bytes, _) = ivec.as_ref().split_at(std::mem::size_of::<usize>());
    bytes.try_into().map(usize::from_be_bytes).ok()
}

const ENTS_OF_TYPE: &str = "ents_of_type";

impl SledDatabase {
    /// Returns sled tree for ent types
    fn ent_type_tree(&self) -> DatabaseResult<sled::Tree> {
        self.0
            .open_tree(ENTS_OF_TYPE)
            .map_err(|e| DatabaseError::Connection {
                source: Box::from(e),
            })
    }

    /// Performs an update on the ent type set of the specified type
    fn update_ent_type_set<F: Fn(EntIdSet) -> EntIdSet>(
        &self,
        r#type: &str,
        f: F,
    ) -> DatabaseResult<()> {
        self.ent_type_tree()?
            .transaction(move |tx_db| {
                let set = match tx_db.get(r#type)? {
                    Some(ivec) => match bincode::deserialize::<EntIdSet>(&ivec) {
                        Ok(x) => x,
                        Err(x) => {
                            sled::transaction::abort(x)?;
                            return Ok(());
                        }
                    },
                    None => HashSet::new(),
                };

                let set = f(set);

                let set_bytes = match bincode::serialize(&set) {
                    Ok(x) => x,
                    Err(x) => {
                        sled::transaction::abort(x)?;
                        return Ok(());
                    }
                };

                tx_db.insert(r#type, set_bytes)?;
                Ok(())
            })
            .map_err(|e| DatabaseError::Connection {
                source: Box::from(e),
            })
    }
}

impl Database for SledDatabase {
    fn get(&self, id: usize) -> DatabaseResult<Option<Ent>> {
        let maybe_ivec = self
            .0
            .get(id_to_ivec(id))
            .map_err(|e| DatabaseError::Connection {
                source: Box::from(e),
            })?;

        maybe_ivec
            .map(|ivec| bincode::deserialize(ivec.as_ref()))
            .transpose()
            .map_err(|e| DatabaseError::CorruptedEnt {
                id,
                source: Box::from(e),
            })
    }

    fn remove(&self, id: usize) -> DatabaseResult<()> {
        // Remove the ent and, if it has an associated schema, we process
        // each of the edges identified in the schema based on deletion attributes
        if let Some(ent) = self
            .0
            .remove(id_to_ivec(id))
            .map_err(|e| DatabaseError::Connection {
                source: Box::from(e),
            })?
            .map(|ivec| bincode::deserialize::<Ent>(ivec.as_ref()))
            .transpose()
            .map_err(|e| DatabaseError::CorruptedEnt {
                id,
                source: Box::from(e),
            })?
        {
            for edge in ent.edges() {
                match edge.deletion_policy() {
                    // If shallow deletion, we only want to remove the connections
                    // back to this ent from the corresponding ents
                    EdgeDeletionPolicy::ShallowDelete => {
                        for edge_id in edge.to_ids() {
                            self.0
                                .transaction(|tx_db| {
                                    let maybe_ivec = tx_db.get(id_to_ivec(id))?;
                                    let result = maybe_ivec
                                        .map(|ivec| bincode::deserialize::<Ent>(ivec.as_ref()))
                                        .transpose()
                                        .map_err(|e| DatabaseError::CorruptedEnt {
                                            id,
                                            source: Box::from(e),
                                        });
                                    match result {
                                        Ok(Some(mut ent)) => {
                                            let _ = ent.remove_ents_from_all_edges(Some(edge_id));
                                            match bincode::serialize(&ent) {
                                                Ok(bytes) => tx_db.insert(id_to_ivec(id), bytes)?,
                                                Err(x) => sled::transaction::abort(
                                                    DatabaseError::CorruptedEnt {
                                                        id: ent.id(),
                                                        source: x,
                                                    },
                                                )?,
                                            };
                                        }
                                        Ok(None) => {}
                                        Err(x) => {
                                            sled::transaction::abort(x)?;
                                        }
                                    };
                                    Ok(())
                                })
                                .map_err(|e| DatabaseError::Connection {
                                    source: Box::from(e),
                                })?;
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
            self.update_ent_type_set(ent.r#type(), |mut set| {
                set.remove(&id);
                set
            })?;
        }

        Ok(())
    }

    fn insert(&self, into_ent: impl Into<Ent>) -> DatabaseResult<()> {
        let ent = into_ent.into();

        // Add our ent's id to the set of ids associated with the ent's type
        self.update_ent_type_set(ent.r#type(), |mut set| {
            set.insert(ent.id());
            set
        })?;

        // Add our ent to the primary database
        let ent_bytes = bincode::serialize(&ent).map_err(|e| DatabaseError::CorruptedEnt {
            id: ent.id(),
            source: Box::from(e),
        })?;
        self.0
            .insert(id_to_ivec(ent.id()), ent_bytes)
            .map_err(|e| DatabaseError::Connection {
                source: Box::from(e),
            })?;

        Ok(())
    }
}

impl KeyValueStoreDatabase for SledDatabase {
    /// Returns ids of all ents stored in the database
    fn ids(&self) -> EntIdSet {
        self.0
            .iter()
            .keys()
            .filter_map(Result::ok)
            .filter_map(ivec_to_id)
            .collect()
    }

    /// Returns true if database contains the provided id
    fn has_id(&self, id: usize) -> bool {
        self.0.contains_key(id_to_ivec(id)).ok().unwrap_or_default()
    }

    /// Returns ids of all ents for the given type
    fn ids_for_type(&self, r#type: &str) -> EntIdSet {
        fn inner(this: &SledDatabase, r#type: &str) -> DatabaseResult<EntIdSet> {
            match this
                .0
                .open_tree(ENTS_OF_TYPE)
                .map_err(|e| DatabaseError::Connection {
                    source: Box::from(e),
                })?
                .get(r#type)
                .map_err(|e| DatabaseError::Connection {
                    source: Box::from(e),
                })? {
                Some(ivec) => match bincode::deserialize::<EntIdSet>(&ivec) {
                    Ok(x) => Ok(x),
                    Err(x) => Err(DatabaseError::Connection {
                        source: Box::from(x),
                    }),
                },
                None => Ok(HashSet::new()),
            }
        }

        inner(self, r#type).ok().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Value};

    fn new_db() -> SledDatabase {
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to create database");
        SledDatabase::new(db)
    }

    #[test]
    fn insert_should_add_a_new_ent_using_its_id() {
        let db = new_db();

        let ent = Ent::new_untyped(999);
        let _ = db.insert(ent).expect("Failed to insert ent");

        let ent = db
            .get(999)
            .expect("Failed to get ent")
            .expect("Ent missing");
        assert_eq!(ent.id(), 999);
    }

    #[test]
    fn insert_should_overwrite_an_existing_ent_with_the_same_id() {
        let db = new_db();

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
        let db = new_db();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_none(), "Unexpectedly acquired ent");

        let _ = db.insert(Ent::new_untyped(999)).unwrap();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_some(), "Unexpectedly missing ent");
    }

    #[test]
    fn remove_should_remove_an_ent_by_id() {
        let db = new_db();

        let _ = db.remove(999).expect("Failed to remove ent");

        let _ = db.insert(Ent::new_untyped(999)).unwrap();
        assert!(db.get(999).unwrap().is_some(), "Failed to set up ent");

        let _ = db.remove(999).expect("Failed to remove ent");
        assert!(db.get(999).unwrap().is_none(), "Did not remove ent");
    }
}
