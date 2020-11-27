use crate::{
    database::{Database, DatabaseError, DatabaseExt, DatabaseResult},
    ent::{
        query::{Condition, EdgeCondition, FieldCondition, Query, TimeCondition},
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
        Condition::Created(cond) => process_created_condition(this, cond, pipeline),
        Condition::LastUpdated(cond) => process_last_updated_condition(this, cond, pipeline),
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
        || pipeline.is_some() && pipeline.unwrap().contains(&id)
    {
        vec![id].into_iter().collect()
    } else {
        HashSet::new()
    }
}

/// If this is part of a pipeline of ids, we filter such that
/// only the ents whose created property pass the condition remain.
/// If this is the start of a pipeline, we check all ents for a
/// created property that passes the condition.
#[inline]
fn process_created_condition(
    this: &InmemoryDatabase,
    cond: &TimeCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_else(|| this.ents.lock().unwrap().keys().copied().collect())
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter(|ent| cond.check(ent.created()))
        .map(|ent| ent.id())
        .collect()
}

/// If this is part of a pipeline of ids, we filter such that
/// only the ents whose last updated property pass the condition remain.
/// If this is the start of a pipeline, we check all ents for a
/// last updated property that passes the condition.
#[inline]
fn process_last_updated_condition(
    this: &InmemoryDatabase,
    cond: &TimeCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_else(|| this.ents.lock().unwrap().keys().copied().collect())
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter(|ent| cond.check(ent.last_updated()))
        .map(|ent| ent.id())
        .collect()
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
/// condition on all ents of that edge. If this is the start
/// of a pipeline, we check ALL ents for an edge with the given name and then
/// perform the given condition.
#[inline]
fn process_edge_condition(
    this: &InmemoryDatabase,
    name: &str,
    condition: &EdgeCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_else(|| this.ents.lock().unwrap().keys().copied().collect())
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter_map(|ent| {
            if let Some(edge) = ent.edge(name) {
                let ids = edge.to_ids().into_iter().collect::<EntIdSet>();
                let id_cnt = ids.len();
                let valid_edge_ids = process_condition(this, condition.condition(), Some(ids));

                match (condition, valid_edge_ids.len()) {
                    (EdgeCondition::Any(_), valid_cnt) if valid_cnt > 0 => Some(ent.id()),
                    (EdgeCondition::Exactly(_, cnt), valid_cnt) if valid_cnt == *cnt => {
                        Some(ent.id())
                    }
                    (EdgeCondition::All(_), valid_cnt) if valid_cnt == id_cnt && valid_cnt > 0 => {
                        Some(ent.id())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

/// If this is part of a pipeline of ids, we check each corresponding
/// ent for a field with the given name and then compare that field's
/// value to our field condition. If the field exists and satisfies the
/// field condition, the id of the ent passes. If this is the start
/// of a pipeline, we check ALL ents for an edge with the given name and then
/// perform the given condition.
#[inline]
fn process_named_field_condition(
    this: &InmemoryDatabase,
    name: &str,
    condition: &FieldCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    pipeline
        .unwrap_or_else(|| this.ents.lock().unwrap().keys().copied().collect())
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
    use crate::{CollectionCondition, Edge, Field, ValueCondition};

    /// Creates a new database with some test entries used throughout
    ///
    /// IDs: 1-3 ~ are type1 with no fields or edges
    /// IDs: 4-6 ~ are type2 with value fields and no edges
    /// IDs: 7-9 ~ are type3 with collection fields and no edges
    /// IDs: 10-12 ~ are type4 with edges to 1-9 and no fields
    fn new_test_database() -> InmemoryDatabase {
        let db = InmemoryDatabase::default();

        // 1-3 have no fields or edges
        let _ = db
            .insert(Ent::from_collections(1, "type1", vec![], vec![]))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(2, "type1", vec![], vec![]))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(3, "type1", vec![], vec![]))
            .unwrap();

        // 4-6 have value fields only
        let _ = db
            .insert(Ent::from_collections(
                4,
                "type2",
                vec![Field::new("a", 1), Field::new("b", 2)],
                vec![],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                5,
                "type2",
                vec![Field::new("a", 3), Field::new("b", 4)],
                vec![],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                6,
                "type2",
                vec![Field::new("a", 5), Field::new("b", 6)],
                vec![],
            ))
            .unwrap();

        // 7-9 have collection fields only
        let _ = db
            .insert(Ent::from_collections(
                7,
                "type3",
                vec![Field::new(
                    "f",
                    Value::from(
                        vec![(String::from("a"), 3), (String::from("b"), 5)]
                            .into_iter()
                            .collect::<HashMap<String, u8>>(),
                    ),
                )],
                vec![],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                8,
                "type3",
                vec![Field::new("f", vec![1, 2])],
                vec![],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                9,
                "type3",
                vec![Field::new(
                    "f",
                    Value::from(
                        vec![
                            (String::from("a"), Value::from(vec![1, 2])),
                            (String::from("b"), Value::from(vec![3, 4])),
                        ]
                        .into_iter()
                        .collect::<HashMap<String, Value>>(),
                    ),
                )],
                vec![],
            ))
            .unwrap();

        // 10-12 have edges only
        let _ = db
            .insert(Ent::from_collections(
                10,
                "type4",
                vec![],
                vec![
                    Edge::new("a", 1),
                    Edge::new("b", vec![3, 4, 5]),
                    Edge::new("c", None),
                ],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                11,
                "type4",
                vec![],
                vec![Edge::new("a", 2), Edge::new("b", vec![1, 2, 3, 4, 5, 6])],
            ))
            .unwrap();
        let _ = db
            .insert(Ent::from_collections(
                12,
                "type4",
                vec![],
                vec![
                    Edge::new("a", 3),
                    Edge::new("b", vec![]),
                    Edge::new("c", Some(8)),
                ],
            ))
            .unwrap();

        db
    }

    fn query_and_assert<Q: Into<Query>>(db: &InmemoryDatabase, query: Q, expected: &[usize]) {
        let query = query.into();
        let results = db
            .find_all(query.clone())
            .expect("Failed to retrieve ents")
            .iter()
            .map(Ent::id)
            .collect::<HashSet<usize>>();
        assert_eq!(
            results,
            expected.into_iter().copied().collect(),
            "{:?}\nExpected: {:?}, Actual: {:?}",
            query,
            expected,
            results
        );
    }

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
        let db = new_test_database();

        // If first condition, will get all ids
        let cond = Condition::Always;
        query_and_assert(&db, cond, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // Otherwise, if part of a chain, will keep any ids in pipeline
        let cond = Condition::HasId(2) & Condition::Always;
        query_and_assert(&db, cond, &[2]);
    }

    #[test]
    fn find_all_should_return_no_ids_if_given_never_condition() {
        let db = new_test_database();

        // If first condition, refuse all ids
        let cond = Condition::Never;
        query_and_assert(&db, cond, &[]);

        // Otherwise, if part of a chain, will block any of its ids
        let cond = Condition::Always & Condition::Never;
        query_and_assert(&db, cond, &[]);
    }

    #[test]
    fn find_all_should_return_ent_with_id_if_given_has_id_condition() {
        let db = new_test_database();

        // If ent with id exists, we expect it to be available
        let cond = Condition::HasId(1);
        query_and_assert(&db, cond, &[1]);

        // If ent with id does not exist, we expect empty
        let cond = Condition::HasId(999);
        query_and_assert(&db, cond, &[]);

        // If we already have ents with ids, this should filter them
        let cond = (Condition::HasId(1) | Condition::HasId(2)) & Condition::HasId(2);
        query_and_assert(&db, cond, &[2]);
    }

    #[test]
    fn find_all_should_return_ents_with_type_if_give_has_type_condition() {
        let db = new_test_database();

        // If ent with type exists, we expect it to be available
        let cond = Condition::HasType(String::from("type1"));
        query_and_assert(&db, cond, &[1, 2, 3]);

        // If ent with type does not exist, we expect empty
        let cond = Condition::HasType(String::from("unknown"));
        query_and_assert(&db, cond, &[]);

        // If we already have ents, this should filter them for that type
        let cond =
            (Condition::HasId(1) | Condition::HasId(8)) & Condition::HasType(String::from("type1"));
        query_and_assert(&db, cond, &[1]);
    }

    #[test]
    fn find_all_should_return_ents_whose_created_property_satisfy_the_time_condition() {
        let db = new_test_database();

        // Re-create all ents with enough time split between them for us to
        // properly test creation time
        for i in 1..=12 {
            let ent = Ent::new_untyped(i);
            db.insert(ent)
                .expect(&format!("Failed to replace ent {}", i));
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Get all ents created after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").created();
        let cond = Condition::Created(TimeCondition::After(time));
        query_and_assert(&db, cond, &[4, 5, 6, 7, 8, 9, 10, 11, 12]);

        let cond = Condition::Always & Condition::Created(TimeCondition::After(time));
        query_and_assert(&db, cond, &[4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // Get all ents created on or after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").created();
        let cond = Condition::Created(TimeCondition::OnOrAfter(time));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        let cond = Condition::Always & Condition::Created(TimeCondition::OnOrAfter(time));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // Get all ents created before our fifth ent
        let time = db.get(5).unwrap().expect("Missing ent 5").created();
        let cond = Condition::Created(TimeCondition::Before(time));
        query_and_assert(&db, cond, &[1, 2, 3, 4]);

        let cond = Condition::Always & Condition::Created(TimeCondition::Before(time));
        query_and_assert(&db, cond, &[1, 2, 3, 4]);

        // Get all ents created on or before our fifth ent
        let time = db.get(5).unwrap().expect("Missing ent 5").created();
        let cond = Condition::Created(TimeCondition::OnOrBefore(time));
        query_and_assert(&db, cond, &[1, 2, 3, 4, 5]);

        let cond = Condition::Always & Condition::Created(TimeCondition::OnOrBefore(time));
        query_and_assert(&db, cond, &[1, 2, 3, 4, 5]);

        // Get all ents created between our third and eighth ent (not including)
        let time_a = db.get(3).unwrap().expect("Missing ent 3").created();
        let time_b = db.get(8).unwrap().expect("Missing ent 8").created();
        let cond = Condition::Created(TimeCondition::Between(time_a, time_b));
        query_and_assert(&db, cond, &[4, 5, 6, 7]);

        let cond = Condition::Always & Condition::Created(TimeCondition::Between(time_a, time_b));
        query_and_assert(&db, cond, &[4, 5, 6, 7]);

        // Get all ents created between our third and eighth ent (including)
        let time_a = db.get(3).unwrap().expect("Missing ent 3").created();
        let time_b = db.get(8).unwrap().expect("Missing ent 8").created();
        let cond = Condition::Created(TimeCondition::OnOrBetween(time_a, time_b));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8]);

        let cond =
            Condition::Always & Condition::Created(TimeCondition::OnOrBetween(time_a, time_b));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn find_all_should_return_ents_whose_last_updated_property_satisfy_the_time_condition() {
        let db = new_test_database();

        // Update all ents with enough time split between them for us to
        // properly test last updated time
        for i in (1..=12).rev() {
            let mut ent = db.get(i).unwrap().expect(&format!("Missing ent {}", i));
            ent.mark_updated();
            db.insert(ent)
                .expect(&format!("Failed to update ent {}", i));
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Get all ents updated after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::After(time));
        query_and_assert(&db, cond, &[1, 2]);

        let cond = Condition::Always & Condition::LastUpdated(TimeCondition::After(time));
        query_and_assert(&db, cond, &[1, 2]);

        // Get all ents updated on or after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::OnOrAfter(time));
        query_and_assert(&db, cond, &[1, 2, 3]);

        let cond = Condition::Always & Condition::LastUpdated(TimeCondition::OnOrAfter(time));
        query_and_assert(&db, cond, &[1, 2, 3]);

        // Get all ents updated before our fifth ent
        let time = db.get(5).unwrap().expect("Missing ent 5").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::Before(time));
        query_and_assert(&db, cond, &[6, 7, 8, 9, 10, 11, 12]);

        let cond = Condition::Always & Condition::LastUpdated(TimeCondition::Before(time));
        query_and_assert(&db, cond, &[6, 7, 8, 9, 10, 11, 12]);

        // Get all ents updated on or before our fifth ent
        let time = db.get(5).unwrap().expect("Missing ent 5").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::OnOrBefore(time));
        query_and_assert(&db, cond, &[5, 6, 7, 8, 9, 10, 11, 12]);

        let cond = Condition::Always & Condition::LastUpdated(TimeCondition::OnOrBefore(time));
        query_and_assert(&db, cond, &[5, 6, 7, 8, 9, 10, 11, 12]);

        // Get all ents updated between our third and eighth ent (not including)
        let time_a = db.get(8).unwrap().expect("Missing ent 8").last_updated();
        let time_b = db.get(3).unwrap().expect("Missing ent 3").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::Between(time_a, time_b));
        query_and_assert(&db, cond, &[4, 5, 6, 7]);

        let cond =
            Condition::Always & Condition::LastUpdated(TimeCondition::Between(time_a, time_b));
        query_and_assert(&db, cond, &[4, 5, 6, 7]);

        // Get all ents updated between our third and eighth ent (including)
        let time_a = db.get(8).unwrap().expect("Missing ent 8").last_updated();
        let time_b = db.get(3).unwrap().expect("Missing ent 3").last_updated();
        let cond = Condition::LastUpdated(TimeCondition::OnOrBetween(time_a, time_b));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8]);

        let cond =
            Condition::Always & Condition::LastUpdated(TimeCondition::OnOrBetween(time_a, time_b));
        query_and_assert(&db, cond, &[3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_both_conditions_if_given_and_condition() {
        let db = new_test_database();

        // If ent passes both conditions, it will be included in return
        let cond = Condition::And(
            Box::from(Condition::HasType(String::from("type2"))),
            Box::from(Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::greater_than(1)),
            )),
        );
        query_and_assert(&db, cond, &[5, 6]);

        // If already have ents in pipeline, they will be filtered by "and"
        let cond = Condition::Always
            & Condition::And(
                Box::from(Condition::HasType(String::from("type2"))),
                Box::from(Condition::Field(
                    String::from("a"),
                    FieldCondition::Value(ValueCondition::greater_than(1)),
                )),
            );
        query_and_assert(&db, cond, &[5, 6]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_either_condition_if_given_or_condition() {
        let db = new_test_database();

        // If ent passes either condition, it will be included in return
        let cond = Condition::Or(
            Box::from(Condition::HasType(String::from("type1"))),
            Box::from(Condition::HasType(String::from("type2"))),
        );
        query_and_assert(&db, cond, &[1, 2, 3, 4, 5, 6]);

        // If already have ents in pipeline, they will be filtered by "or"
        let cond = Condition::Always
            & Condition::Or(
                Box::from(Condition::HasType(String::from("type1"))),
                Box::from(Condition::HasType(String::from("type2"))),
            );
        query_and_assert(&db, cond, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_only_one_of_two_conditions_if_given_xor_condition() {
        let db = new_test_database();

        // If ent passes one of two conditions, it will be included in return
        let cond = Condition::Xor(
            Box::from(Condition::HasType(String::from("type1"))),
            Box::from(Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::greater_than(1)),
            )),
        );
        query_and_assert(&db, cond, &[1, 2, 3, 5, 6]);

        // If already have ents in pipeline, they will be filtered by "xor"
        let cond = Condition::Always
            & Condition::Xor(
                Box::from(Condition::HasType(String::from("type1"))),
                Box::from(Condition::Field(
                    String::from("a"),
                    FieldCondition::Value(ValueCondition::greater_than(1)),
                )),
            );
        query_and_assert(&db, cond, &[1, 2, 3, 5, 6]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_failing_a_condition_wrapped_in_not_condition() {
        let db = new_test_database();

        // If ent passes not condition, it will be included in return
        let cond = Condition::Not(Box::from(Condition::HasType(String::from("type1"))));
        query_and_assert(&db, cond, &[4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // If already have ents in pipeline, they will be filtered by "not"
        let cond = Condition::HasType(String::from("type2"))
            & Condition::Not(Box::from(Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::greater_than(1)),
            )));
        query_and_assert(&db, cond, &[4]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_value_equal_to_condition() {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("a"),
            FieldCondition::Value(ValueCondition::equal_to(3)),
        );
        query_and_assert(&db, cond, &[5]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type2"))
            & Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::equal_to(3)),
            );
        query_and_assert(&db, cond, &[5]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_value_greater_than_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("a"),
            FieldCondition::Value(ValueCondition::greater_than(1)),
        );
        query_and_assert(&db, cond, &[5, 6]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type2"))
            & Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::greater_than(1)),
            );
        query_and_assert(&db, cond, &[5, 6]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_value_less_than_condition() {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("a"),
            FieldCondition::Value(ValueCondition::less_than(5)),
        );
        query_and_assert(&db, cond, &[4, 5]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type2"))
            & Condition::Field(
                String::from("a"),
                FieldCondition::Value(ValueCondition::less_than(5)),
            );
        query_and_assert(&db, cond, &[4, 5]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_any_collection_value_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionValue(CollectionCondition::any(
                ValueCondition::greater_than(1),
            )),
        );
        query_and_assert(&db, cond, &[7, 8]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionValue(CollectionCondition::any(
                    ValueCondition::greater_than(1),
                )),
            );
        query_and_assert(&db, cond, &[7, 8]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_exactly_n_collection_values_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionValue(CollectionCondition::exactly(
                ValueCondition::greater_than(1),
                2,
            )),
        );
        query_and_assert(&db, cond, &[7]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionValue(CollectionCondition::exactly(
                    ValueCondition::greater_than(1),
                    2,
                )),
            );
        query_and_assert(&db, cond, &[7]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_all_collection_values_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionValue(CollectionCondition::all(
                ValueCondition::greater_than(1),
            )),
        );
        query_and_assert(&db, cond, &[7]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionValue(CollectionCondition::all(
                    ValueCondition::greater_than(1),
                )),
            );
        query_and_assert(&db, cond, &[7]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_len_collection_values_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionValue(CollectionCondition::len(ValueCondition::equal_to(2))),
        );
        query_and_assert(&db, cond, &[7, 8, 9]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionValue(CollectionCondition::len(
                    ValueCondition::equal_to(2),
                )),
            );
        query_and_assert(&db, cond, &[7, 8, 9]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_any_collection_key_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionKey(CollectionCondition::any(ValueCondition::greater_than(
                "a",
            ))),
        );
        query_and_assert(&db, cond, &[7, 9]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionKey(CollectionCondition::any(
                    ValueCondition::greater_than("a"),
                )),
            );
        query_and_assert(&db, cond, &[7, 9]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_exactly_n_collection_keys_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionKey(CollectionCondition::exactly(
                ValueCondition::greater_than("a"),
                1,
            )),
        );
        query_and_assert(&db, cond, &[7, 9]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionKey(CollectionCondition::exactly(
                    ValueCondition::greater_than("a"),
                    1,
                )),
            );
        query_and_assert(&db, cond, &[7, 9]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_all_collection_keys_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionKey(CollectionCondition::all(ValueCondition::less_than("c"))),
        );
        query_and_assert(&db, cond, &[7, 9]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionKey(CollectionCondition::all(ValueCondition::less_than(
                    "c",
                ))),
            );
        query_and_assert(&db, cond, &[7, 9]);
    }

    #[test]
    fn find_all_should_return_ents_that_match_have_a_field_passing_the_len_collection_keys_condition(
    ) {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let cond = Condition::Field(
            String::from("f"),
            FieldCondition::CollectionKey(CollectionCondition::len(ValueCondition::equal_to(2))),
        );
        query_and_assert(&db, cond, &[7, 9]);

        // If already have ents in pipeline, they will be filtered by "field"
        let cond = Condition::HasType(String::from("type3"))
            & Condition::Field(
                String::from("f"),
                FieldCondition::CollectionKey(CollectionCondition::len(ValueCondition::equal_to(
                    2,
                ))),
            );
        query_and_assert(&db, cond, &[7, 9]);
    }

    #[test]
    fn find_all_should_return_ents_that_pass_the_any_edge_ent_condition() {
        let db = new_test_database();

        // If ent's edge passes condition, it will be included in return
        let cond = Condition::Edge(String::from("a"), EdgeCondition::any(Condition::HasId(2)));
        query_and_assert(&db, cond, &[11]);

        // If already have ents in pipeline, they will be filtered by "edge"
        let cond = Condition::HasType(String::from("type4"))
            & Condition::Edge(String::from("a"), EdgeCondition::any(Condition::HasId(2)));
        query_and_assert(&db, cond, &[11]);
    }

    #[test]
    fn find_all_should_return_ents_that_pass_the_exactly_n_edge_ent_condition() {
        let db = new_test_database();

        // If ent's edge passes condition, it will be included in return
        let cond = Condition::Edge(
            String::from("b"),
            EdgeCondition::exactly(Condition::HasId(2) | Condition::HasId(3), 2),
        );
        query_and_assert(&db, cond, &[11]);

        // If already have ents in pipeline, they will be filtered by "edge"
        let cond = Condition::HasType(String::from("type4"))
            & Condition::Edge(
                String::from("b"),
                EdgeCondition::exactly(Condition::HasId(2) | Condition::HasId(3), 2),
            );
        query_and_assert(&db, cond, &[11]);
    }

    #[test]
    fn find_all_should_return_ents_that_pass_the_all_edge_ent_condition() {
        let db = new_test_database();

        // If ent's edge passes condition, it will be included in return
        let cond = Condition::Edge(
            String::from("b"),
            EdgeCondition::all(Condition::HasId(3) | Condition::HasId(4) | Condition::HasId(5)),
        );
        query_and_assert(&db, cond, &[10]);

        // If already have ents in pipeline, they will be filtered by "edge"
        let cond = Condition::HasType(String::from("type4"))
            & Condition::Edge(
                String::from("b"),
                EdgeCondition::all(Condition::HasId(3) | Condition::HasId(4) | Condition::HasId(5)),
            );
        query_and_assert(&db, cond, &[10]);
    }
}
