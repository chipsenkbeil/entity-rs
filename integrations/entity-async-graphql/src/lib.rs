mod ent;
mod filter;

pub use ent::*;
pub use filter::*;

#[cfg(feature = "derive")]
pub use entity_async_graphql_macros::*;

#[cfg(test)]
mod tests {
    use super::*;

    use async_graphql::{value, Context, EmptyMutation, EmptySubscription, Object, Schema};
    use entity::{Database, DatabaseRc, Edge, Field, Id, UntypedEnt, Value};
    use entity_inmemory::InmemoryDatabase;
    use entity_sled::SledDatabase;
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

            struct TestQuery;

            #[Object]
            impl TestQuery {
                async fn ent<'ctx>(
                    &self,
                    ctx: &'ctx Context<'_>,
                    id: Option<Id>,
                    filter: Option<GqlEntFilter>,
                ) -> async_graphql::Result<Vec<GqlDynEnt>> {
                    let db = ctx.data::<DatabaseRc>()?;

                    if let Some(id) = id {
                        db.get_all(vec![id]).map(|x| x.into_iter().map(GqlDynEnt::from).collect())
                            .map_err(|x| async_graphql::Error::new(x.to_string()))
                    } else if let Some(filter) = filter {
                        db.find_all(filter.into()).map(|x| x.into_iter().map(GqlDynEnt::from).collect())
                            .map_err(|x| async_graphql::Error::new(x.to_string()))
                    } else {
                        Err(async_graphql::Error::new("Must provide one argument"))
                    }
                }
            }

            #[test]
            fn supports_ent_trait_object_as_output_object() {
                let schema = Schema::build(TestQuery, EmptyMutation, EmptySubscription)
                    .data(DatabaseRc::new(Box::new(new_test_database())))
                    .finish();
                let input = r#"
                    { 
                        ent(id: 1) { 
                            id 
                        } 
                    }
                "#;
                let response = futures::executor::block_on(schema.execute(input.trim()));
                assert_eq!(
                    response.data,
                    value!({
                        "ent": [
                            { "id": 1 },
                        ],
                    })
                );
            }

            #[test]
            fn supports_filtering() {
                let schema = Schema::build(TestQuery, EmptyMutation, EmptySubscription)
                    .data(DatabaseRc::new(Box::new(new_test_database())))
                    .finish();
                let input = r#"
                    { 
                        ent(filter: { id: { equals: 1 } }) { 
                            id 
                        } 
                    }
                "#;
                let response = futures::executor::block_on(schema.execute(input.trim()));
                assert_eq!(
                    response.data,
                    value!({
                        "ent": [
                            { "id": 1 },
                        ],
                    })
                );
            }
        };
    }

    mod inmemory {
        use super::*;

        impl_tests!(InmemoryDatabase, InmemoryDatabase::default());
    }

    mod sled {
        use super::*;

        impl_tests!(
            SledDatabase,
            SledDatabase::new(::sled::Config::new().temporary(true).open().unwrap())
        );
    }
}
