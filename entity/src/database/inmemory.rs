use crate::{
    database::{Database, DatabaseError, DatabaseExt, DatabaseResult},
    ent::{
        query::{Condition, EdgeCondition, FieldCondition, Query},
        EdgeDeletionPolicy, Ent, Value,
    },
    IEnt,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

type EntIdSet = HashSet<usize>;

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
pub struct InmemoryDatabase {
    /// Primary ent storage
    ents: Arc<Mutex<HashMap<usize, Ent>>>,

    /// Type matching from specific ents to all ids of those ents
    ents_of_type: Arc<Mutex<HashMap<String, EntIdSet>>>,
}

impl Default for InmemoryDatabase {
    /// Creates a new, empty database entry
    fn default() -> Self {
        Self {
            ents: Arc::new(Mutex::new(HashMap::new())),
            ents_of_type: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Database for InmemoryDatabase {
    fn get(&self, id: usize) -> DatabaseResult<Option<Ent>> {
        Ok(self.ents.lock().unwrap().get(&id).map(Clone::clone))
    }

    fn remove(&self, id: usize) -> DatabaseResult<()> {
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
        }

        Ok(())
    }

    fn insert(&self, into_ent: impl Into<Ent>) -> DatabaseResult<()> {
        let ent = into_ent.into();

        // Add our ent's id to the set of ids associated with the ent's type
        self.ents_of_type
            .lock()
            .unwrap()
            .entry(ent.r#type().to_string())
            .or_insert_with(HashSet::new)
            .insert(ent.id());

        // Add our ent to the primary database
        self.ents.lock().unwrap().insert(ent.id(), ent);

        Ok(())
    }
}

impl DatabaseExt for InmemoryDatabase {
    fn get_all(&self, ids: impl IntoIterator<Item = usize>) -> DatabaseResult<Vec<Ent>> {
        ids.into_iter()
            .filter_map(|id| self.get(id).transpose())
            .collect()
    }

    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Ent>> {
        // Find the ids that match the query and convert them into the
        // underlying ents
        process_condition(self, query.as_condition(), None)
            .into_iter()
            .filter_map(|id| self.get(id).transpose())
            .collect()
    }
}

/// Will take a condition and determine the ids of the ents that pass its criteria
#[inline]
fn process_condition(
    this: &InmemoryDatabase,
    condition: &Condition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    match condition {
        Condition::Always => process_always_condition(this, pipeline),
        Condition::Never => process_never_condition(this, pipeline),
        Condition::And(a, b) => process_and_condition(this, a, b, pipeline),
        Condition::Or(a, b) => process_or_condition(this, a, b, pipeline),
        Condition::Xor(a, b) => process_xor_condition(this, a, b, pipeline),
        Condition::Not(cond) => process_not_condition(this, cond, pipeline),
        Condition::HasId(id) => process_has_id_condition(this, *id, pipeline),
        Condition::HasType(r#type) => process_has_type_condition(this, r#type, pipeline),
        Condition::Field(name, cond) => process_named_field_condition(this, name, cond, pipeline),
        Condition::Edge(name, cond) => process_edge_condition(this, name, cond, pipeline),
    }
}

/// If this is part of a pipeline if ids, we pass them all along,
/// else this is the first step in a pipeline, we get all ids
/// available in the entire database
#[inline]
fn process_always_condition(this: &InmemoryDatabase, pipeline: Option<EntIdSet>) -> EntIdSet {
    match pipeline {
        Some(ids) => ids,
        None => this.ents.lock().unwrap().keys().copied().collect(),
    }
}

/// Regardless of the pipeline state, no ids will pass
#[inline]
fn process_never_condition(_this: &InmemoryDatabase, _pipeline: Option<EntIdSet>) -> EntIdSet {
    EntIdSet::new()
}

/// Pipeline of (a -> b), where a does the first filtering and
/// then b does the second filtering, leaving only those that
/// pass both a AND b
#[inline]
fn process_and_condition(
    this: &InmemoryDatabase,
    a: &Condition,
    b: &Condition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    let a_output = process_condition(this, a, pipeline);
    process_condition(this, b, Some(a_output))
}

/// Pipeline of (a -> c, b -> c), where both a and b filter on
/// the same initial pipeline and feed their results into c,
/// which takes the union of them, maintaining anything that
/// was in a OR b
#[inline]
fn process_or_condition(
    this: &InmemoryDatabase,
    a: &Condition,
    b: &Condition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    let a_output = process_condition(this, a, pipeline.clone());
    let b_output = process_condition(this, b, pipeline);
    a_output.union(&b_output).copied().collect()
}

/// Pipeline of (a -> c, b -> c), where both a and b filter on
/// the same initial pipeline and feed their results into c,
/// which takes the difference of them, maintaining anything that
/// was in a XOR b (only a or only b)
#[inline]
fn process_xor_condition(
    this: &InmemoryDatabase,
    a: &Condition,
    b: &Condition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    let a_output = process_condition(this, a, pipeline.clone());
    let b_output = process_condition(this, b, pipeline);

    a_output.symmetric_difference(&b_output).copied().collect()
}

/// If this is part of a pipeline of ids, we filter such that
/// only ids that don't match the condition are maintained
///
/// If this is the start of a pipeline, we want to filter all
/// out any ids produced against all ids in the database
#[inline]
fn process_not_condition(
    this: &InmemoryDatabase,
    condition: &Condition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    if let Some(ids) = pipeline {
        let all_ids = ids.clone();
        let ids_to_remove = process_condition(this, condition, Some(ids));
        all_ids.difference(&ids_to_remove).copied().collect()
    } else {
        let all_ids = this
            .ents
            .lock()
            .unwrap()
            .keys()
            .copied()
            .collect::<EntIdSet>();
        let ids_to_remove = process_condition(this, condition, None);
        all_ids.difference(&ids_to_remove).copied().collect()
    }
}

/// If this is part of a pipeline of ids, we filter such that
/// only this id remains. If this is the start of a pipeline,
/// we check if the id exists in our database and return it
/// if it does. Otherwise, regardless of the pipeline, nothing
/// passes.
#[inline]
fn process_has_id_condition(
    this: &InmemoryDatabase,
    id: usize,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    if (pipeline.is_none() && this.ents.lock().unwrap().contains_key(&id))
        || pipeline.unwrap().contains(&id)
    {
        vec![id].into_iter().collect()
    } else {
        HashSet::new()
    }
}

/// If this is part of a pipeline of ids, we need to check the
/// type of each associated ent to only include those that have
/// the matching type. If this is the start of a pipeline, we
/// return all ids for the given type. Otherwise, regardless of
/// the pipeline, nothing passes.
#[inline]
fn process_has_type_condition(
    this: &InmemoryDatabase,
    r#type: &str,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    if let Some(ids) = pipeline {
        ids.into_iter()
            .filter_map(|id| {
                this.ents
                    .lock()
                    .unwrap()
                    .get(&id)
                    .filter(|ent| ent.r#type() == r#type)
                    .map(|ent| ent.id())
            })
            .collect()
    } else {
        this.ents_of_type
            .lock()
            .unwrap()
            .get(r#type)
            .cloned()
            .unwrap_or_default()
    }
}

/// If this is part of a pipeline of ids, we check each corresponding
/// ent for an edge with the given name and then perform the given
/// condition on all ents of that edge. If all en If this is the start
/// of a pipeline, nothing passes.
#[inline]
fn process_edge_condition(
    this: &InmemoryDatabase,
    name: &str,
    condition: &EdgeCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .flat_map(|ent| {
            if let Some(edge) = ent.edge(name) {
                let ids = edge.to_ids().into_iter().collect::<EntIdSet>();
                let id_cnt = ids.len();
                let valid_edge_ids = process_condition(this, condition.condition(), Some(ids));

                match (condition, valid_edge_ids.len()) {
                    (EdgeCondition::Any(_), _) => valid_edge_ids,
                    (EdgeCondition::Exactly(_, cnt), valid_cnt) if valid_cnt == *cnt => {
                        valid_edge_ids
                    }
                    (EdgeCondition::All(_), valid_cnt) if valid_cnt == id_cnt => valid_edge_ids,
                    _ => HashSet::new(),
                }
            } else {
                HashSet::new()
            }
        })
        .collect()
}

/// If this is part of a pipeline of ids, we check each corresponding
/// ent for a field with the given name and then compare that field's
/// value to our field condition. If the field exists and satisfies the
/// field condition, the id of the ent passes. If this is the start
/// of a pipeline, nothing passes.
#[inline]
fn process_named_field_condition(
    this: &InmemoryDatabase,
    name: &str,
    condition: &FieldCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter_map(|ent| match lookup_ent_field_value(&ent, &name).ok() {
            Some(value) if condition.check(value) => Some(ent.id()),
            _ => None,
        })
        .collect()
}

/// Looks up the value of a field on an ent
#[inline]
fn lookup_ent_field_value<'a>(ent: &'a Ent, name: &str) -> Result<&'a Value, DatabaseError> {
    let value = ent
        .field_value(name)
        .ok_or_else(|| DatabaseError::MissingField {
            name: name.to_string(),
        })?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ent::Field;

    #[test]
    fn insert_should_add_a_new_ent_using_its_id() {
        let db = InmemoryDatabase::default();

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
    }

    #[test]
    fn get_all_should_return_all_ents_with_associated_ids() {
        let db = InmemoryDatabase::default();

        let _ = db.insert(Ent::new_untyped(1)).unwrap();
        let _ = db.insert(Ent::new_untyped(2)).unwrap();
        let _ = db.insert(Ent::new_untyped(3)).unwrap();

        let results = db
            .get_all(vec![1, 2, 3])
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(results, [1, 2, 3].iter().copied().collect());

        let results = db
            .get_all(vec![1, 3])
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(results, [1, 3].iter().copied().collect());

        let results = db
            .get_all(vec![2, 3, 4, 5, 6, 7, 8])
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(results, [2, 3].iter().copied().collect());
    }

    #[test]
    fn find_all_should_return_all_ids_if_given_always_condition() {
        let db = InmemoryDatabase::default();

        let _ = db.insert(Ent::new_empty(1, "type1")).unwrap();
        let _ = db.insert(Ent::new_untyped(2)).unwrap();
        let _ = db.insert(Ent::new_untyped(3)).unwrap();

        let results = db
            .find_all(Query::new(Condition::Always))
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(results, [1, 2, 3].iter().copied().collect());
    }

    #[test]
    fn find_all_should_return_no_ids_if_given_never_condition() {
        let db = InmemoryDatabase::default();

        let _ = db.insert(Ent::new_empty(1, "type1")).unwrap();
        let _ = db.insert(Ent::new_untyped(2)).unwrap();
        let _ = db.insert(Ent::new_untyped(3)).unwrap();

        let results = db
            .find_all(Query::new(Condition::Never))
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(results, [].iter().copied().collect());
    }
}
