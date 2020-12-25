#[cfg(feature = "inmemory_db")]
mod inmemory;

#[cfg(feature = "inmemory_db")]
pub use inmemory::InmemoryDatabase;

#[cfg(feature = "sled_db")]
mod sled_db;
#[cfg(feature = "sled_db")]
pub use sled_db::SledDatabase;

use crate::{
    database::{Database, DatabaseResult},
    Filter, IEnt, Id, Query,
};
use std::collections::HashSet;

type EntIdSet = HashSet<Id>;

/// Represents a key-value store database that performs synchronous insertion,
/// retrieval, and removal. It provides blanket support for `DatabaseExt` to
/// perform complex operations.
pub trait KeyValueDatabase: Database {
    /// Returns ids of all ents stored in the database
    fn ids(&self) -> EntIdSet;
}

pub struct KeyValueDatabaseExecutor<'a, T: KeyValueDatabase>(&'a T);

impl<'a, T: KeyValueDatabase> From<&'a T> for KeyValueDatabaseExecutor<'a, T> {
    fn from(db: &'a T) -> Self {
        Self(db)
    }
}

impl<'a, T: KeyValueDatabase> KeyValueDatabaseExecutor<'a, T> {
    pub fn get_all(&self, ids: Vec<Id>) -> DatabaseResult<Vec<Box<dyn IEnt>>> {
        ids.into_iter()
            .filter_map(|id| self.0.get(id).transpose())
            .collect()
    }

    pub fn find_all(&self, query: Query) -> DatabaseResult<Vec<Box<dyn IEnt>>> {
        let mut pipeline: Option<EntIdSet> = None;

        for filter in query {
            pipeline
                .get_or_insert_with(|| self.0.ids())
                .retain(|id| filter_id(self.0, id, &filter));
        }

        pipeline
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| self.0.get(id).transpose())
            .collect()
    }
}

fn filter_id<D: KeyValueDatabase>(db: &D, id: &Id, filter: &Filter) -> bool {
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
    }
}

fn with_ent<D: KeyValueDatabase, F: Fn(Box<dyn IEnt>) -> bool>(db: &D, id: &Id, f: F) -> bool {
    db.get(*id)
        .map(|maybe_ent| maybe_ent.map(f).unwrap_or_default())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Ent, Field, Predicate as P, TypedPredicate as TP, Value};
    use std::collections::HashMap;

    macro_rules! impl_tests {
        ($db_type:ty, $new_db:expr) => {
            /// Creates a new database with some test entries used throughout
            ///
            /// IDs: 1-3 ~ are type1 with no fields or edges
            /// IDs: 4-6 ~ are type2 with value fields and no edges
            /// IDs: 7-9 ~ are type3 with collection fields and no edges
            /// IDs: 10-12 ~ are type4 with edges to 1-9 and no fields
            fn new_test_database() -> $db_type {
                let db = $new_db;

                // 1-3 have no fields or edges
                let _ = db
                    .insert(Box::from(Ent::from_collections(1, "type1", vec![], vec![])))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(2, "type1", vec![], vec![])))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(3, "type1", vec![], vec![])))
                    .unwrap();

                // 4-6 have value fields only
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        4,
                        "type2",
                        vec![Field::new("a", 1), Field::new("b", 2)],
                        vec![],
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        5,
                        "type2",
                        vec![Field::new("a", 3), Field::new("b", 4)],
                        vec![],
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        6,
                        "type2",
                        vec![Field::new("a", 5), Field::new("b", 6)],
                        vec![],
                    )))
                    .unwrap();

                // 7-9 have collection fields only
                let _ = db
                    .insert(Box::from(Ent::from_collections(
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
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        8,
                        "type3",
                        vec![Field::new("f", vec![1, 2])],
                        vec![],
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
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
                    )))
                    .unwrap();

                // 10-12 have edges only
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        10,
                        "type4",
                        vec![],
                        vec![
                            Edge::new("a", 1),
                            Edge::new("b", vec![3, 4, 5]),
                            Edge::new("c", None),
                        ],
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        11,
                        "type4",
                        vec![],
                        vec![Edge::new("a", 2), Edge::new("b", vec![1, 2, 3, 4, 5, 6])],
                    )))
                    .unwrap();
                let _ = db
                    .insert(Box::from(Ent::from_collections(
                        12,
                        "type4",
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

            fn query_and_assert<Q: Into<Query>, T: KeyValueDatabase>(
                db: &T,
                query: Q,
                expected: &[Id],
            ) {
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
            fn get_all_should_return_all_ents_with_associated_ids() {
                let db = $new_db;

                let _ = db.insert(Box::from(Ent::new_untyped(1))).unwrap();
                let _ = db.insert(Box::from(Ent::new_untyped(2))).unwrap();
                let _ = db.insert(Box::from(Ent::new_untyped(3))).unwrap();

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

                // If ent with type exists, we expect it to be available
                let q = Query::default().where_type(TP::equals(String::from("type1")));
                query_and_assert(&db, q, &[1, 2, 3]);

                // If ent with type does not exist, we expect empty
                let q = Query::default().where_type(TP::equals(String::from("unknown")));
                query_and_assert(&db, q, &[]);

                // If already in a pipeline, should only filter the existing ids
                let q = Query::default()
                    .where_id(TP::equals(1) | TP::equals(2) | TP::equals(4))
                    .where_type(TP::equals(String::from("type1")));
                query_and_assert(&db, q, &[1, 2]);
            }

            #[test]
            fn find_all_should_support_filtering_by_created_timestamp() {
                let db = new_test_database();

                // Re-create all ents with enough time split between them for us to
                // properly test creation time
                for i in 1..=12 {
                    let ent = Ent::new_untyped(i);
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
                    use crate::DatabaseExt;
                    let mut ent = db
                        .get_typed::<Ent>(i)
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
                    .where_created(TP::greater_than(time));
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
                    .where_type(TP::equals(String::from("type2")))
                    .where_field("a", P::equals(3));
                query_and_assert(&db, q, &[5]);
            }

            #[test]
            fn find_all_should_support_filtering_by_edge() {
                let db = new_test_database();

                todo!();
            }
        };
    }

    #[cfg(feature = "inmemory_db")]
    mod inmemory {
        use super::*;

        impl_tests!(InmemoryDatabase, InmemoryDatabase::default());
    }

    #[cfg(feature = "sled_db")]
    mod sled_db {
        use super::*;

        impl_tests!(
            SledDatabase,
            SledDatabase::new(sled::Config::new().temporary(true).open().unwrap())
        );
    }
}
