use crate::{
    database::{Database, DatabaseError, DatabaseExt, DatabaseResult},
    ent::{
        query::{Condition, EdgeCondition, FieldCondition, Query},
        Ent, Value, ValueType,
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
    /// Primary database, using a simple, locked hashmap
    database: Arc<Mutex<HashMap<usize, Ent>>>,

    /// Type matching from specific ents to all ids of those ents
    ents_of_type: Arc<Mutex<HashMap<String, EntIdSet>>>,
}

impl Default for InmemoryDatabase {
    /// Creates a new, empty database entry
    fn default() -> Self {
        Self {
            database: Arc::new(Mutex::new(HashMap::new())),
            ents_of_type: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Database for InmemoryDatabase {
    fn get(&self, id: usize) -> DatabaseResult<Option<Ent>> {
        Ok(self.database.lock().unwrap().get(&id).map(Clone::clone))
    }

    fn remove(&self, id: usize, adjust_edges: bool) -> DatabaseResult<()> {
        if let Some(ent) = self.database.lock().unwrap().remove(&id) {
            todo!("Process adjusting edges of ents pointing to this one");
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
        self.database.lock().unwrap().insert(ent.id(), ent);

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
        Condition::Field(name, cond) => process_field_condition(this, name, cond, pipeline),
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
        None => this.database.lock().unwrap().keys().copied().collect(),
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
            .database
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
    if (pipeline.is_none() && this.database.lock().unwrap().contains_key(&id))
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
                this.database
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
/// ent for a field with the given name and then compare that field's
/// value to our field condition. If the field exists and satisfies the
/// field condition, the id of the ent passes. If this is the start
/// of a pipeline, nothing passes.
#[inline]
fn process_field_condition(
    this: &InmemoryDatabase,
    name: &str,
    condition: &FieldCondition,
    pipeline: Option<EntIdSet>,
) -> EntIdSet {
    // TODO: Bubble up errors rather than filtering them out
    pipeline
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter_map(|ent| {
            let maybe_v = lookup_ent_field_value(&ent, &name, condition.value().to_type()).ok();
            match (condition, maybe_v) {
                (FieldCondition::EqualTo(v), Some(ent_v)) if ent_v == v => Some(ent.id()),
                (FieldCondition::GreaterThan(v), Some(ent_v)) if ent_v > v => Some(ent.id()),
                (FieldCondition::LessThan(v), Some(ent_v)) if ent_v < v => Some(ent.id()),
                _ => None,
            }
        })
        .collect()
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
    // TODO: Bubble up errors rather than filtering them out
    pipeline
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| this.get(id).ok().flatten())
        .filter_map(|ent| {
            todo!("Need to handle three types of edges and three types of edge values")
        })
        .collect()
}

/// Looks up the value of a field on an ent
#[inline]
fn lookup_ent_field_value<'a>(
    ent: &'a Ent,
    name: &str,
    r#type: ValueType,
) -> Result<&'a Value, DatabaseError> {
    let value = ent
        .field_value(name)
        .ok_or_else(|| DatabaseError::MissingField {
            name: name.to_string(),
        })?;

    if value.is_type(r#type.clone()) {
        Ok(value)
    } else {
        Err(DatabaseError::WrongType {
            expected: r#type,
            actual: value.to_type(),
        })
    }
}
