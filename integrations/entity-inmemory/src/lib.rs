use entity::{
    Database, DatabaseError, DatabaseResult, EdgeDeletionPolicy, Ent, Filter, Id, IdAllocator,
    Predicate, Primitive, Query, Value, EPHEMERAL_ID,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

type EntIdSet = HashSet<Id>;

/// Represents an in-memory database that performs synchronous insertion,
/// retrieval, and removal. If the feature `serde` is enabled, this database
/// can be serialized and deserialized.
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct InmemoryDatabase {
    /// Primary ent storage
    ents: Mutex<HashMap<Id, Box<dyn Ent>>>,

    /// Type matching from specific ents to all ids of those ents
    ents_of_type: Mutex<HashMap<String, EntIdSet>>,

    /// Id allocator for ents
    alloc: Mutex<IdAllocator>,
}

impl InmemoryDatabase {
    /// Creates a new instance of an in-memory database
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InmemoryDatabase {
    /// Creates a new, empty database entry
    fn default() -> Self {
        Self {
            ents: Mutex::new(HashMap::new()),
            ents_of_type: Mutex::new(HashMap::new()),
            alloc: Mutex::new(IdAllocator::new()),
        }
    }
}

impl Database for InmemoryDatabase {
    fn get_all(&self, ids: Vec<Id>) -> DatabaseResult<Vec<Box<dyn Ent>>> {
        ids.into_iter()
            .filter_map(|id| self.get(id).transpose())
            .collect()
    }

    fn find_all(&self, query: Query) -> DatabaseResult<Vec<Box<dyn Ent>>> {
        let mut pipeline: Option<EntIdSet> = None;

        for filter in query {
            let mut_pipeline = pipeline.get_or_insert_with(|| prefill_ids(self, &filter));

            // If our filter is the special IntoEdge case, we don't want to
            // actually filter out ids but rather transform them into the ids
            // of their edge
            match filter {
                Filter::IntoEdge(name) => {
                    pipeline = Some(
                        mut_pipeline
                            .iter()
                            .flat_map(|id| {
                                self.get(*id)
                                    .map(|maybe_ent| {
                                        maybe_ent
                                            .and_then(|ent| {
                                                ent.edge(&name).map(|edge| edge.to_ids())
                                            })
                                            .unwrap_or_default()
                                    })
                                    .unwrap_or_default()
                            })
                            .collect(),
                    )
                }
                // Otherwise, the filter is a traditional case where we will
                // strip out ids by the filter
                f => {
                    mut_pipeline.retain(|id| filter_id(self, id, &f));
                }
            }
        }

        pipeline
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| self.get(id).transpose())
            .collect()
    }

    fn get(&self, id: Id) -> DatabaseResult<Option<Box<dyn Ent>>> {
        let maybe_ent = self
            .ents
            .lock()
            .unwrap()
            .get(&id)
            .map(|ent| dyn_clone::clone_box(ent.as_ref()));

        // If we found an ent without a database connection, attempt to fill
        // it in with the global database if it exists
        Ok(if let Some(mut ent) = maybe_ent {
            if !ent.is_connected() {
                ent.connect(entity::global::db());
            }
            Some(ent)
        } else {
            None
        })
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
                                for mut edge in ent.edges() {
                                    let _ = edge.value_mut().remove_ids(Some(edge_id));
                                    let name = edge.name().to_string();
                                    let _ = ent.update_edge(&name, edge.into_value());
                                }
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

    fn insert(&self, mut ent: Box<dyn Ent>) -> DatabaseResult<Id> {
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

        // Clear any cache before saving the ent
        ent.clear_cache();

        // Update the ent's last_updated to be the current time
        ent.mark_updated().map_err(|e| DatabaseError::Other {
            source: Box::from(e),
        })?;

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

impl InmemoryDatabase {
    /// Returns ids of all ents stored in the database
    pub fn ids(&self) -> EntIdSet {
        self.ents.lock().unwrap().keys().copied().collect()
    }

    /// Returns true if database contains the provided id
    pub fn has_id(&self, id: Id) -> bool {
        self.ents.lock().unwrap().contains_key(&id)
    }

    /// Returns ids of all ents for the given type
    pub fn ids_for_type(&self, r#type: &str) -> EntIdSet {
        self.ents_of_type
            .lock()
            .unwrap()
            .get(r#type)
            .cloned()
            .unwrap_or_default()
    }
}

/// Called once when first beginning to filter to determine which ent ids
/// to start with based on the leading filter
///
/// 1. If lead filter by id equality, will only include those ids that match
///    the predicate
/// 2. If lead filter by type equality, will only include those ids that equal
///    the type (or many types if wrapped in Or)
/// 3. Any other variation of id/type filter or other kind of filter will
///    result in the more expensive pulling of all ids
fn prefill_ids(db: &InmemoryDatabase, filter: &Filter) -> EntIdSet {
    fn from_id_predicate(
        db: &InmemoryDatabase,
        p: &Predicate,
        mut ids: EntIdSet,
    ) -> Option<EntIdSet> {
        match p {
            Predicate::Equals(Value::Primitive(Primitive::Number(id))) => Some({
                ids.insert(id.to_usize());
                ids
            }),
            Predicate::Or(list) => list.iter().fold(Some(ids), |ids, p| match ids {
                Some(ids) => from_id_predicate(db, p, ids),
                None => None,
            }),
            _ => None,
        }
    }

    fn from_type_predicate(
        db: &InmemoryDatabase,
        p: &Predicate,
        mut ids: EntIdSet,
    ) -> Option<EntIdSet> {
        match p {
            Predicate::Equals(Value::Text(t)) => Some({
                ids.extend(db.ids_for_type(t));
                ids
            }),
            Predicate::Or(list) => list.iter().fold(Some(ids), |ids, p| match ids {
                Some(ids) => from_type_predicate(db, p, ids),
                None => None,
            }),
            _ => None,
        }
    }

    match filter {
        // If leading with id, support Equals and Or(Equals(...), ...) for
        // specific ids; otherwise, too hard to figure out so we pull in all ids
        Filter::Id(p) => {
            from_id_predicate(db, p.as_untyped(), EntIdSet::new()).unwrap_or_else(|| db.ids())
        }

        // If leading with type, support Equals and Or(Equals(...), ...) for
        // specific ids; otherwise, too hard to figure out so we pull in all ids
        Filter::Type(p) => {
            from_type_predicate(db, p.as_untyped(), EntIdSet::new()).unwrap_or_else(|| db.ids())
        }

        // Otherwise, currently no cached/indexed way to look up (yet)
        // TODO: Support database field indexing so equality of a field can
        //       be used for faster id lookup; do the same for timestamp fields
        _ => db.ids(),
    }
}

fn filter_id(db: &InmemoryDatabase, id: &Id, filter: &Filter) -> bool {
    match filter {
        Filter::Id(p) => p.check(*id),
        Filter::Type(p) => with_ent(db, id, |ent| p.check(ent.r#type().to_string())),
        Filter::Created(p) => with_ent(db, id, |ent| p.check(ent.created())),
        Filter::LastUpdated(p) => with_ent(db, id, |ent| p.check(ent.last_updated())),
        Filter::Field(name, p) => with_ent(db, id, |ent| match ent.field(name) {
            Some(value) => p.check(&value),
            None => false,
        }),
        Filter::Edge(name, f) => with_ent(db, id, |ent| match ent.edge(name) {
            Some(edge) => edge.to_ids().iter().any(|id| filter_id(db, id, f)),
            None => false,
        }),

        // NOTE: Logically, this should be impossible to reach since we only
        //       call this when we know that the filter is not a transformation
        Filter::IntoEdge(_) => unreachable!("Bug: Transformation in filter"),
    }
}

fn with_ent<F: Fn(Box<dyn Ent>) -> bool>(db: &InmemoryDatabase, id: &Id, f: F) -> bool {
    db.get(*id)
        .map(|maybe_ent| maybe_ent.map(f).unwrap_or_default())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::{Predicate as P, TypedPredicate as TP, *};

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
            .insert(Box::from(UntypedEnt::from_collections(1, vec![], vec![])))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(2, vec![], vec![])))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(3, vec![], vec![])))
            .unwrap();

        // 4-6 have value fields only
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                4,
                vec![Field::new("a", 1), Field::new("b", 2)],
                vec![],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                5,
                vec![Field::new("a", 3), Field::new("b", 4)],
                vec![],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                6,
                vec![Field::new("a", 5), Field::new("b", 6)],
                vec![],
            )))
            .unwrap();

        // 7-9 have collection fields only
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                7,
                vec![Field::new(
                    "f",
                    Value::from(
                        vec![(String::from("a"), 3), (String::from("b"), 5)]
                            .into_iter()
                            .collect::<HashMap<String, u8>>(),
                    ),
                )],
                vec![],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                8,
                vec![Field::new("f", vec![1, 2])],
                vec![],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                9,
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
            )))
            .unwrap();

        // 10-12 have edges only
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                10,
                vec![],
                vec![
                    Edge::new("a", 1),
                    Edge::new("b", vec![3, 4, 5]),
                    Edge::new("c", None),
                ],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                11,
                vec![],
                vec![Edge::new("a", 2), Edge::new("b", vec![1, 2, 3, 4, 5, 6])],
            )))
            .unwrap();
        let _ = db
            .insert(Box::from(UntypedEnt::from_collections(
                12,
                vec![],
                vec![
                    Edge::new("a", 3),
                    Edge::new("b", vec![]),
                    Edge::new("c", Some(8)),
                ],
            )))
            .unwrap();

        db
    }

    fn query_and_assert<Q: Into<Query>>(db: &InmemoryDatabase, query: Q, expected: &[Id]) {
        let query = query.into();
        let results = db
            .find_all(query.clone())
            .expect("Failed to retrieve ents")
            .iter()
            .map(|ent| ent.id())
            .collect::<HashSet<Id>>();
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
    fn insert_should_replace_ephemeral_id_with_allocator_id() {
        let db = InmemoryDatabase::default();

        let ent = UntypedEnt::empty_with_id(EPHEMERAL_ID);
        let id = db.insert(Box::from(ent)).expect("Failed to insert ent");
        assert_ne!(id, EPHEMERAL_ID);

        let ent = db.get(id).expect("Failed to get ent").expect("Ent missing");
        assert_eq!(ent.id(), id);
    }

    #[test]
    fn insert_should_update_the_last_updated_time_with_the_current_time() {
        let db = InmemoryDatabase::default();

        let ent = UntypedEnt::empty_with_id(EPHEMERAL_ID);
        let last_updated = ent.last_updated();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let id = db.insert(Box::from(ent)).expect("Failed to insert ent");
        let ent = db.get(id).expect("Failed to get ent").expect("Ent missing");
        assert!(ent.last_updated() > last_updated);
    }

    #[test]
    fn insert_should_add_a_new_ent_using_its_id() {
        let db = InmemoryDatabase::default();

        let ent = UntypedEnt::empty_with_id(999);
        let id = db.insert(Box::from(ent)).expect("Failed to insert ent");
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

        let ent = UntypedEnt::from_collections(999, vec![Field::new("field1", 3)], vec![]);
        let _ = db.insert(Box::from(ent)).expect("Failed to insert ent");

        let ent = db
            .get(999)
            .expect("Failed to get ent")
            .expect("Ent missing");
        assert_eq!(ent.field("field1").expect("Field missing"), Value::from(3));
    }

    #[test]
    fn insert_should_reset_all_computed_field_caches_to_none() {
        let db = InmemoryDatabase::default();

        // Verify that a computed field is reset to None
        let ent = UntypedEnt::from_collections(
            999,
            vec![Field::new_with_attributes(
                "field1",
                Some(3),
                vec![FieldAttribute::Computed],
            )],
            vec![],
        );
        let _ = db.insert(Box::from(ent)).expect("Failed to insert ent");

        let ent = db
            .get(999)
            .expect("Failed to get ent")
            .expect("Ent missing");
        assert_eq!(
            ent.field("field1").expect("Field missing"),
            Value::Optional(None)
        );
    }

    #[test]
    fn get_should_return_an_ent_by_id() {
        use entity::DatabaseRc;
        let db = InmemoryDatabase::default();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_none(), "Unexpectedly acquired ent");

        let _ = db
            .insert(Box::from(UntypedEnt::empty_with_id(999)))
            .unwrap();

        let result = db.get(999).expect("Failed to get ent");
        assert!(result.is_some(), "Unexpectedly missing ent");
        assert!(
            !result.unwrap().is_connected(),
            "Ent unexpectedly connected to database"
        );

        // Verify that if a global database is available, it will populate
        let db = DatabaseRc::new(Box::new(db));
        entity::global::with_db_from_rc(DatabaseRc::clone(&db), || {
            let result = db.get(999).expect("Failed to get ent");
            assert!(result.is_some(), "Unexpectedly missing ent");
            assert!(
                result.unwrap().is_connected(),
                "Ent unexpectedly not connected to database"
            );
        });
    }

    #[test]
    fn remove_should_remove_an_ent_by_id() {
        let db = InmemoryDatabase::default();

        let _ = db.remove(999).expect("Failed to remove ent");

        let _ = db
            .insert(Box::from(UntypedEnt::empty_with_id(999)))
            .unwrap();
        assert!(db.get(999).unwrap().is_some(), "Failed to set up ent");

        let _ = db.remove(999).expect("Failed to remove ent");
        assert!(db.get(999).unwrap().is_none(), "Did not remove ent");

        // Id allocator should indicate that id has been freed
        assert_eq!(db.alloc.lock().unwrap().freed(), &[999]);
    }

    #[test]
    fn get_all_should_return_all_ents_with_associated_ids() {
        let db = InmemoryDatabase::default();

        let _ = db.insert(Box::from(UntypedEnt::empty_with_id(1))).unwrap();
        let _ = db.insert(Box::from(UntypedEnt::empty_with_id(2))).unwrap();
        let _ = db.insert(Box::from(UntypedEnt::empty_with_id(3))).unwrap();

        let results = db
            .get_all(vec![1, 2, 3])
            .expect("Failed to retrieve ents")
            .iter()
            .map(|ent| ent.id())
            .collect::<HashSet<Id>>();
        assert_eq!(results, [1, 2, 3].iter().copied().collect());

        let results = db
            .get_all(vec![1, 3])
            .expect("Failed to retrieve ents")
            .iter()
            .map(|ent| ent.id())
            .collect::<HashSet<Id>>();
        assert_eq!(results, [1, 3].iter().copied().collect());

        let results = db
            .get_all(vec![2, 3, 4, 5, 6, 7, 8])
            .expect("Failed to retrieve ents")
            .iter()
            .map(|ent| ent.id())
            .collect::<HashSet<Id>>();
        assert_eq!(results, [2, 3].iter().copied().collect());
    }

    #[test]
    fn find_all_should_return_no_ents_by_default() {
        let db = new_test_database();

        let q = Query::default();
        query_and_assert(&db, q, &[]);
    }

    #[test]
    fn find_all_should_support_filtering_by_id() {
        let db = new_test_database();

        // If ent with id exists, we expect it to be available
        let q = Query::default().where_id(TP::equals(1));
        query_and_assert(&db, q, &[1]);

        // If ent with either id exists, we expect it to be available
        let q = Query::default().where_id(TP::equals(1) | TP::equals(2));
        query_and_assert(&db, q, &[1, 2]);

        // If ent with id does not exist, we expect empty
        let q = Query::default().where_id(TP::equals(999));
        query_and_assert(&db, q, &[]);

        // If already in a pipeline, should only filter the existing ids
        let q = Query::default()
            .where_id(TP::equals(1) | TP::equals(2))
            .where_id(TP::equals(1) | TP::equals(3));
        query_and_assert(&db, q, &[1]);
    }

    #[test]
    fn find_all_should_support_filtering_by_type() {
        let db = new_test_database();
        let _ = db.insert(Box::from(TestEnt::new(20))).unwrap();
        let _ = db.insert(Box::from(TestEnt::new(21))).unwrap();
        let _ = db.insert(Box::from(TestEnt::new(22))).unwrap();

        // If ent with type exists, we expect it to be available
        let ts = <UntypedEnt as EntType>::type_str();
        let q = Query::default().where_type(TP::equals(ts.to_string()));
        query_and_assert(&db, q, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // If ent with either type exists, we expect it to be available
        let q = Query::default().where_type(TP::or(vec![
            TP::equals(<UntypedEnt as EntType>::type_str().to_string()),
            TP::equals(<TestEnt as EntType>::type_str().to_string()),
        ]));
        query_and_assert(&db, q, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 20, 21, 22]);

        // If ent with type does not exist, we expect empty
        let q = Query::default().where_type(TP::equals(String::from("unknown")));
        query_and_assert(&db, q, &[]);

        // If already in a pipeline, should only filter the existing ids
        let q = Query::default()
            .where_id(TP::equals(1) | TP::equals(2) | TP::equals(4))
            .where_type(TP::equals(ts.to_string()));
        query_and_assert(&db, q, &[1, 2, 4]);
    }

    #[test]
    fn find_all_should_support_filtering_by_created_timestamp() {
        let db = new_test_database();

        // Re-create all ents with enough time split between them for us to
        // properly test creation time
        for i in 1..=12 {
            let ent = UntypedEnt::empty_with_id(i);
            db.insert(Box::from(ent))
                .expect(&format!("Failed to replace ent {}", i));
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Get all ents created after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").created();
        let q = Query::default().where_created(TP::greater_than(time));
        query_and_assert(&db, q, &[4, 5, 6, 7, 8, 9, 10, 11, 12]);

        // If already in a pipeline, should only filter the existing ids
        let time = db.get(3).unwrap().expect("Missing ent 3").created();
        let q = Query::default()
            .where_id(TP::less_than(8))
            .where_created(TP::greater_than(time));
        query_and_assert(&db, q, &[4, 5, 6, 7]);
    }

    #[test]
    fn find_all_should_support_filtering_by_last_updated_timestamp() {
        let db = new_test_database();

        // Update all ents with enough time split between them for us to
        // properly test last updated time
        for i in (1..=12).rev() {
            use entity::DatabaseExt;
            let mut ent = db
                .get_typed::<UntypedEnt>(i)
                .unwrap()
                .expect(&format!("Missing ent {}", i));
            ent.mark_updated().unwrap();
            db.insert(Box::from(ent))
                .expect(&format!("Failed to update ent {}", i));
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Get all ents updated after our third ent
        let time = db.get(3).unwrap().expect("Missing ent 3").last_updated();
        let q = Query::default().where_last_updated(TP::greater_than(time));
        query_and_assert(&db, q, &[1, 2]);

        // If already in a pipeline, should only filter the existing ids
        let time = db.get(3).unwrap().expect("Missing ent 3").created();
        let q = Query::default()
            .where_id(TP::equals(2))
            .where_last_updated(TP::greater_than(time));
        query_and_assert(&db, q, &[2]);
    }

    #[test]
    fn find_all_should_support_filtering_by_field() {
        let db = new_test_database();

        // If ent's field passes condition, it will be included in return
        let q = Query::default().where_field("a", P::equals(3));
        query_and_assert(&db, q, &[5]);

        // If already have ents in pipeline, they will be filtered by "field"
        let q = Query::default()
            .where_id(TP::equals(4) | TP::equals(6))
            .where_field("a", P::greater_than(1));
        query_and_assert(&db, q, &[6]);
    }

    #[test]
    fn find_all_should_support_filtering_by_edge() {
        let db = new_test_database();

        // If ent's edge passes condition, it will be included in return
        let q = Query::default().where_edge("a", Filter::Id(TP::equals(3)));
        query_and_assert(&db, q, &[12]);

        // If already have ents in pipeline, they will be filtered by "edge"
        let q = Query::default()
            .where_id(TP::equals(10) | TP::equals(12))
            .where_edge("a", Filter::Id(TP::always()));
        query_and_assert(&db, q, &[10, 12]);
    }

    #[test]
    fn find_all_should_support_transforming_into_edge() {
        let db = new_test_database();

        // Will take the ids of each ent with the given edge and use
        // them going forward; in this example, ents #10 and #11 have
        // overlapping ids for edge b
        let q = Query::default().where_into_edge("b");
        query_and_assert(&db, q, &[1, 2, 3, 4, 5, 6]);

        // If already have ents in pipeline, their edge's ids will
        // be used specifically; in this example, ent #12 has no ents
        // for edge b
        let q = Query::default()
            .where_id(TP::equals(10) | TP::equals(12))
            .where_into_edge("b");
        query_and_assert(&db, q, &[3, 4, 5]);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    struct TestEnt(Id);

    impl TestEnt {
        pub fn new(id: Id) -> Self {
            Self(id)
        }
    }

    impl EntType for TestEnt {
        fn type_data() -> EntTypeData {
            EntTypeData::Concrete {
                ty: concat!(module_path!(), "::TestEnt"),
            }
        }
    }

    #[cfg_attr(feature = "serde-1", typetag::serde)]
    impl Ent for TestEnt {
        fn id(&self) -> Id {
            self.0
        }

        fn set_id(&mut self, id: Id) {
            self.0 = id;
        }

        fn r#type(&self) -> &str {
            Self::type_str()
        }

        fn created(&self) -> u64 {
            0
        }

        fn last_updated(&self) -> u64 {
            0
        }

        fn mark_updated(&mut self) -> Result<(), EntMutationError> {
            Ok(())
        }

        fn field_definitions(&self) -> Vec<FieldDefinition> {
            Vec::new()
        }

        fn field_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn field(&self, _name: &str) -> Option<Value> {
            None
        }

        fn update_field(&mut self, name: &str, _value: Value) -> Result<Value, EntMutationError> {
            Err(EntMutationError::NoField {
                name: name.to_string(),
            })
        }

        fn edge_definitions(&self) -> Vec<EdgeDefinition> {
            Vec::new()
        }

        fn edge_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn edge(&self, _name: &str) -> Option<EdgeValue> {
            None
        }

        fn update_edge(
            &mut self,
            name: &str,
            _value: EdgeValue,
        ) -> Result<EdgeValue, EntMutationError> {
            Err(EntMutationError::NoEdge {
                name: name.to_string(),
            })
        }

        fn connect(&mut self, _database: WeakDatabaseRc) {}

        fn disconnect(&mut self) {}

        fn is_connected(&self) -> bool {
            false
        }

        fn load_edge(&self, _name: &str) -> DatabaseResult<Vec<Box<dyn Ent>>> {
            Err(DatabaseError::Disconnected)
        }

        fn clear_cache(&mut self) {}

        fn refresh(&mut self) -> DatabaseResult<()> {
            Err(DatabaseError::Disconnected)
        }

        fn commit(&mut self) -> DatabaseResult<()> {
            Err(DatabaseError::Disconnected)
        }

        fn remove(&self) -> DatabaseResult<bool> {
            Err(DatabaseError::Disconnected)
        }
    }
}
